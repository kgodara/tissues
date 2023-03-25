#![allow(dead_code)]
#[allow(unused_imports)]

#[macro_use]
extern crate lazy_static;


use std::io;
use std::fs;
use std::sync::{Arc, atomic::{Ordering}};

mod app;
mod linear;
mod ui;
mod constants;
mod util;
mod command;

mod components;

use crate::components::{
    InputComponent,
};

use crate::linear::{
    client::{LinearClient},
    config::LinearConfig,
};

use app::{ Route, InputMode };


extern crate dotenv;
use dotenv::dotenv;

use tokio::{sync::Mutex as tMutex,};

// use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
// use tui::{ backend::TermionBackend, Terminal, };
use crossterm::{
    event::{ DisableMouseCapture, EnableMouseCapture },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{ backend::CrosstermBackend, Terminal };



use util::{
    // event::{Event, Events},
    event_crossterm::{Event, Events},
    loader::{ LOADER_STATE_MAX },
};

use crate::constants::{ SCROLL_TICK_MAX };

#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

use std::fs::File;

use command::{ Command,
                get_cmd,

                exec_editor_focus_cmd,
                exec_editor_input_cmd,
                exec_editor_delete_cmd,
                exec_editor_move_forward_cmd,
                exec_editor_move_back_cmd,
                exec_editor_submit_cmd,
                exec_editor_exit_cmd,

                exec_delete_cmd,
                exec_select_view_panel_cmd,

                exec_refresh_view_panel_cmd,
                exec_expand_issue_cmd,

                exec_select_dashboard_view_list_cmd,
                exec_select_custom_view_select_cmd,
                exec_open_issue_op_interface_cmd,
                exec_move_back_cmd,
                exec_confirm_cmd,
                exec_scroll_down_cmd,
                exec_scroll_up_cmd,
};


fn get_platform() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    println!("Buffer: {}", buffer);
    Ok(buffer)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenv().ok();

    let log_remove_result = fs::remove_file("rust_cli.log");

    match log_remove_result {
        Ok(_) => {},
        Err(x) => {
            match x.kind() {
                io::ErrorKind::NotFound => {},
                _ => panic!(),
            }
        }
    }

    WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("rust_cli.log").unwrap()).unwrap();

    // Create default app state
    let mut app = app::App::default();

    // Attempt to load access token, if successful bypass access token entry route
    {
        // Access Token found, continue
        match LinearConfig::load_config() {
            Some(config) => {
                // with_config() can return Err() if token file contains non visible ASCII chars (32-127)
                match LinearClient::with_config(config) {
                    Ok(client) => {
                        *app.viewer_obj_render.lock().unwrap() = client.config.viewer_object.clone();

                        app.input_mode = InputMode::Normal;
                        app.linear_client = Arc::new(tMutex::new(Some(client)));
                        app.change_route(Route::ActionSelect);
                    },
                    Err(_) => {
                        app.input_mode = InputMode::Edit;
                        app.active_input = InputComponent::TokenEntry;
                    }
                }
            },
            None => {
                app.input_mode = InputMode::Edit;
                app.active_input = InputComponent::TokenEntry;
            }
        }
    }

    // Terminal initialization

    // let stdout = io::stdout().into_raw_mode()?;
    // let stdout = MouseTerminal::from(stdout);
    // let stdout = AlternateScreen::from(stdout);
    // let backend = TermionBackend::new(stdout);
    // let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut events = Events::new();

    terminal.clear()?;

    let mut tick_idx = 0u64;
    let mut cmd_option: Option<Command>;

    loop {

        terminal.draw(|f| {
            let cur_route: Route = app.route.lock().unwrap().clone();
            match cur_route {
                Route::ConfigInterface => {
                    ui::draw_config_interface(f, &mut app);
                },
                Route::ActionSelect => {
                    ui::draw_action_select(f, &mut app);
                },
                Route::DashboardViewDisplay => {
                    ui::draw_dashboard_view_config(f, &mut app);
                }
            };
        })?;

        // Change Route if some other task changed it
        if app.change_route.load(Ordering::Relaxed) {
            app.change_route.store(false, Ordering::Relaxed);
            let cur_route: Route = app.route.lock().unwrap().clone();
            app.change_route(cur_route);
        }

        let event_next = events.next()?;

        match event_next {
            Event::Input(input) => {

                let cur_route: Route = app.route.lock().unwrap().clone();

                // Update Command String / Get Command to apply
                cmd_option = get_cmd(&mut app.cmd_str, input, & cur_route, &app.input_mode);
                info!("cmd_option: {:?}", cmd_option);

                if let Some(cmd) = cmd_option {
                    tick_idx = 0;
                    app.cmd_str.clear();

                    // Execute Command
                    match cmd {
                        Command::Quit => {
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            terminal.show_cursor()?;
                            break;
                        },


                        // Editor-related Commands
                        Command::EditorEnter => {
                            exec_editor_focus_cmd(&mut app, &mut events);
                        },
                        Command::EditorInput(ch) => {
                            exec_editor_input_cmd(&mut app, &ch);
                        },
                        Command::EditorDelete => {
                            exec_editor_delete_cmd(&mut app);
                        },
                        Command::EditorMoveForward => {
                            exec_editor_move_forward_cmd(&mut app);            
                        },
                        Command::EditorMoveBackward => {
                            exec_editor_move_back_cmd(&mut app);
                        },
                        Command::EditorSubmit => {
                            exec_editor_submit_cmd(&mut app, &mut events);
                        },
                        Command::EditorExit => {
                            exec_editor_exit_cmd(&mut app, &mut events);
                        },


                        Command::Delete => {
                            exec_delete_cmd(&mut app).await;
                        },
                        Command::SelectViewPanel(idx) => {
                            // linear_dashboard_view_panel_selected
                            exec_select_view_panel_cmd(&mut app, idx);
                        },

                        Command::RefreshViewPanel => {
                            exec_refresh_view_panel_cmd(&mut app);
                        },
                        Command::ExpandIssue => {
                            exec_expand_issue_cmd(&mut app);
                        },

                        Command::SelectDashboardViewList => {
                            exec_select_dashboard_view_list_cmd(&mut app);
                        },
                        Command::SelectCustomViewSelect => {
                            exec_select_custom_view_select_cmd(&mut app);
                        },
                        Command::OpenIssueOpInterface(x) => {
                            exec_open_issue_op_interface_cmd(&mut app, x);
                        },
                        Command::MoveBack => {
                            exec_move_back_cmd(&mut app);
                        },
                        Command::Confirm => {
                            exec_confirm_cmd(&mut app).await;
                        },
                        Command::ScrollDown => {
                            exec_scroll_down_cmd(&mut app);
                        },
                        Command::ScrollUp => {
                            exec_scroll_up_cmd(&mut app);
                        },
                    };
                }
                else {
                    tick_idx += 1;
                    if tick_idx >= 4 {
                        info!("Clearing Command String");
                        app.cmd_str.clear();
                    }
                }
            },
            Event::Tick => {
                // info!("tick_idx: {}", tick_idx);
                // info!("Tick event - app.cmd_str: {:?}", app.cmd_str);

                if app.loader_tick == (LOADER_STATE_MAX-1) { app.loader_tick = 0; }
                else { app.loader_tick += 1; }

                if app.scroll_tick == ( SCROLL_TICK_MAX-1 ) { app.scroll_tick = 0; }
                else { app.scroll_tick += 1; }

                // avoid overflow
                if tick_idx < 100 {
                    tick_idx += 1;
                }
            },
            Event::Quit => {
                disable_raw_mode()?;
                execute!(
                    terminal.backend_mut(),
                    LeaveAlternateScreen,
                    DisableMouseCapture
                )?;
                terminal.show_cursor()?;
                break;
            }
        };
    }

    Ok(())
}