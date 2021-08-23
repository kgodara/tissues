#![allow(dead_code)]
#[allow(unused_imports)]

#[macro_use]
extern crate lazy_static;


use std::io;
use std::fs;

mod app;
mod graphql;
mod linear;
mod ui;
mod constants;
mod util;
mod errors;
mod command;
mod network;

mod components;

use app::Route as Route;

use serde_json::Value;

extern crate dotenv;

use dotenv::dotenv;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    Terminal,
};

use util::{
    event::{Event, Events},
    loader::{ LOADER_STATE_MAX },
};

#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

use std::fs::File;

use command::{ Command,
                get_cmd,
                exec_add_cmd,
                exec_replace_cmd,
                exec_delete_cmd,
                exec_select_view_panel_cmd,
                exec_select_dashboard_view_list_cmd,
                exec_select_custom_view_select_cmd,
                exec_open_linear_workflow_state_selection_cmd,
                exec_move_back_cmd,
                exec_confirm_cmd,
                exec_scroll_down_cmd,
                exec_scroll_up_cmd,
};
use network::IOEvent;



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

    // Create a new channel with a capacity of at most 8.
    let (tx, mut rx) = mpsc::channel(8);

    let _manager = tokio::spawn(async move {
        // Establish a connection to the server
        // let mut client = client::connect("127.0.0.1:6379").await.unwrap();
    
        // Start receiving messages
        while let Some(cmd) = rx.recv().await {

            info!("Manager received IOEvent::{:?}", cmd);
            match cmd {
                IOEvent::LoadLinearTeamTimeZones { linear_config, resp } => {
                    let tz_list_option = linear::load_linear_team_timezones(linear_config).await;
                    info!("LoadLinearTeamTimeZones data: {:?}", tz_list_option);

                    let _ = resp.send(tz_list_option);
                },
                IOEvent::LoadCustomViews { linear_config, linear_cursor, resp } => {
                    let option_stateful = components::linear_custom_view_select::LinearCustomViewSelect::load_custom_views(linear_config, Some(linear_cursor)).await;
                    info!("LoadCustomViews data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);
                },
                IOEvent::LoadViewIssues { linear_config, view, team_tz_lookup, tz_offset_lookup, issue_data, view_loader, resp } => {
                    // let option_stateful = linear::view_resolver::get_issues_from_view(&view, linear_config).await;
                    let issue_list = linear::view_resolver::optimized_view_issue_fetch(&view, view_loader,
                                                                                        team_tz_lookup,
                                                                                        tz_offset_lookup,
                                                                                        issue_data,
                                                                                        linear_config).await;
                    info!("LoadViewIssues data: {:?}", issue_list);

                    let _ = resp.send(issue_list);
                },
                IOEvent::LoadLinearTeams { api_key, resp } => {
                    let option_stateful = components::linear_team_select::LinearTeamSelectState::load_teams(api_key).await;
                    info!("LoadLinearTeams data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);

                    // client.get(&key).await;
                },
                IOEvent::LoadWorkflowStates { linear_config, team, resp } => {

                    let option_stateful = components::
                                            linear_workflow_state_display::
                                                LinearWorkflowStateDisplayState::load_workflow_states_by_team(linear_config, &team).await;
                    info!("LoadWorkflowStates data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);
                },
                IOEvent::UpdateIssueWorkflowState { linear_config, issue_id, workflow_state_id, resp } => {

                    let mut issue_update_variables = serde_json::Map::new();

                    issue_update_variables.insert(String::from("issueId"), Value::String(issue_id));
                    issue_update_variables.insert(String::from("stateId"), Value::String(workflow_state_id));

                    let option_stateful = linear::client::LinearClient::update_issue_workflow_state(linear_config, issue_update_variables).await;

                    info!("UpdateIssueWorkflowState data: {:?}", option_stateful);


                    let _ = resp.send(option_stateful.ok());
                }
            }
        }
    });



    // Create default app state
    let mut app = app::App::default();


    // Load Linear Team Timezones for all teams within organization and add to app.team_tz_map

    let tx2 = tx.clone();

    let linear_config_dup = app.linear_client.config.clone();
    let team_tz_map_handle = app.team_tz_map.clone();
    let team_tz_load_done_handle = app.team_tz_load_done.clone();

    let _time_zone_load = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();


        let cmd = IOEvent::LoadLinearTeamTimeZones {    linear_config: linear_config_dup,
                                                        resp: resp_tx };
        
        tx2.send(cmd).await.unwrap();

        let res = resp_rx.await.ok();

        info!("LoadLinearTeamTimeZones IOEvent returned: {:?}", res);

        let mut team_tz_map_lock = team_tz_map_handle.lock().unwrap();
        let mut team_tz_load_done_lock = team_tz_load_done_handle.lock().unwrap();

        if let Some(id_tz_pairs) = res {
            for pair in id_tz_pairs.iter() {
                team_tz_map_lock.insert(pair.0.clone(), pair.1.clone());
            }
            *team_tz_load_done_lock = true;
        }
    });


    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    terminal.clear()?;

    let mut tick_idx = 0u64;
    // loader_tick is a looping index for loader_state
    let mut loader_tick = 0u16;
    let mut cmd_option: Option<Command>;

    loop {

        terminal.draw(|mut f| match app.route {
            Route::ActionSelect => {
              ui::draw_action_select(&mut f, &mut app, &loader_tick);
            },
            Route::DashboardViewDisplay => {
                ui::draw_dashboard_view_display(&mut f, &mut app);
            }
        })?;
        let event_next = events.next()?;

        match event_next {
            Event::Input(input) => {
                // Update Command String / Get Command to apply
                cmd_option = get_cmd(&mut app.cmd_str, input, & app.route);
                info!("cmd_option: {:?}", cmd_option);

                if let Some(cmd) = cmd_option {
                    tick_idx = 0;
                    app.cmd_str.clear();

                    // Execute Command
                    match cmd {
                        Command::Quit => {
                            break;
                        },
                        /*
                        Command::Add => {
                            exec_add_cmd(&mut app).await;
                        },
                        Command::Replace => {
                            exec_replace_cmd(&mut app).await;
                        },
                        */
                        Command::Delete => {
                            exec_delete_cmd(&mut app).await;
                        },
                        Command::SelectViewPanel(idx) => {
                            // linear_dashboard_view_panel_selected
                            exec_select_view_panel_cmd(&mut app, idx).await;
                        },
                        Command::SelectDashboardViewList => {
                            exec_select_dashboard_view_list_cmd(&mut app);
                        },
                        Command::SelectCustomViewSelect => {
                            exec_select_custom_view_select_cmd(&mut app);
                        },
                        Command::OpenLinearWorkflowStateSelection => {
                            exec_open_linear_workflow_state_selection_cmd(&mut app, &tx);
                        },
                        Command::MoveBack => {
                            exec_move_back_cmd(&mut app, &tx);
                        },
                        Command::Confirm => {
                            exec_confirm_cmd(&mut app, &tx).await;
                        },
                        Command::ScrollDown => {
                            exec_scroll_down_cmd(&mut app, &tx);
                        },
                        Command::ScrollUp => {
                            exec_scroll_up_cmd(&mut app);
                        }
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
                
                if loader_tick == (LOADER_STATE_MAX-1) { loader_tick = 0; }
                else { loader_tick += 1; }

                // avoid overflow
                if tick_idx < 100 {
                    tick_idx += 1;
                }
            },
        };
    }

    Ok(())
}