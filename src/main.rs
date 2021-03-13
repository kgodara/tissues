#![allow(dead_code)]

use std::io;
use std::fs;

use serde_json;

mod graphql;
mod linear;
mod ui;
mod util;
mod errors;
mod command;

mod components;

extern crate dotenv;

use dotenv::dotenv;
use std::env;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};

use util::{
    event::{Event, Events},
    StatefulList,
};

#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

use std::fs::File;

use command::Command;



fn get_platform() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    println!("Buffer: {}", buffer);
    Ok(buffer)
}

enum Route {
    ActionSelect,
    TeamSelect,
    LinearInterface,
}

/*
enum InputMode {
    Normal,
    Editing,
}
*/

/// App holds the state of the application
pub struct App<'a> {
    // current route
    route: Route,
    /// Current value of the input box
    input: String,
    // LinearClient
    linear_client: linear::client::LinearClient,

    // Linear Team Select State
    linear_team_select_state: components::linear_team_select::LinearTeamSelectState,

    // Selected Linear Team
    linear_selected_team_idx: Option<usize>,

    // Linear Issue Display State
    linear_issue_display_state: components::linear_issue_display::LinearIssueDisplayState,

    // Available actions
    actions: util::StatefulList<&'a str>,
}

impl<'a> Default for App<'a> {
    fn default() -> App<'a> {
        App {
            route: Route::ActionSelect,
            input: String::new(),

            linear_client: linear::client::LinearClient::default(),

            linear_team_select_state: components::linear_team_select::LinearTeamSelectState::default(),
            // Null
            linear_selected_team_idx: None,
 
            linear_issue_display_state: components::linear_issue_display::LinearIssueDisplayState::default(),

            actions: util::StatefulList::with_items(vec![
                "Create Issue",
                "Test",
            ]),
        }
    }
}

impl<'a> App<'a> {


    async fn change_route(&mut self, route: Route, tx: &tokio::sync::mpsc::Sender<Command>) {
        match route {
            Route::ActionSelect => {},
            Route::TeamSelect => {

                let tx2 = tx.clone();

                let api_key = self.linear_client.config.api_key.clone();

                let team_data_handle = self.linear_team_select_state.teams_data.clone();


                let t1 = tokio::spawn(async move {

                    let (resp_tx, resp_rx) = oneshot::channel();

                    let cmd = Command::LoadLinearTeams { api_key: api_key, resp: resp_tx };
                    tx2.send(cmd).await.unwrap();

                    let res = resp_rx.await.ok();

                    info!("LoadLinearTeams Command returned: {:?}", res);

                    let mut team_data_lock = team_data_handle.lock().unwrap();

                    match res {
                        Some(x) => {
                            *team_data_lock = x;
                        }
                        None => {},
                    }

                    info!("New self.linear_team_select_state.teams_data: {:?}", team_data_lock);
                });


                // self.linear_team_select_state.load_teams(&mut self.linear_client).await;
            },
            
            Route::LinearInterface => {

                let tx3 = tx.clone();
                
                let api_key = self.linear_client.config.api_key.clone();

                let team_data_handle = self.linear_team_select_state.teams_data.clone();
                let team_issue_handle = self.linear_issue_display_state.issue_table_data.clone();

                info!("Linear Interface Loading Process Begun");

                match self.linear_selected_team_idx {
                    Some(idx) => {
                        // Arc<Option<T>> -> Option<&T>
                        match &*team_data_handle.lock().unwrap() {
                            Some(data) => {
                                
                                // self.linear_issue_display_state.load_issues(&self.linear_client, &data.items[idx]).await;

                                let team = data[idx].clone();

                                match team {
                                    serde_json::Value::Null => return,
                                    _ => {},
                                }

                                let t2 = tokio::spawn(async move {
                    
                                    let (resp2_tx, resp2_rx) = oneshot::channel();
                
                                    let cmd = Command::LoadLinearIssues { api_key: api_key, selected_team: team, resp: resp2_tx };
                                    tx3.send(cmd).await.unwrap();
                
                                    let res = resp2_rx.await.ok();
                
                                    info!("LoadLinearIssues Command returned: {:?}", res);
                
                                    let mut issue_data_lock = team_issue_handle.lock().unwrap();
                
                                    match res {
                                        Some(x) => {
                                            *issue_data_lock = x;
                                        }
                                        None => {},
                                    }
                
                                    // info!("New self.linear_team_select_state.teams_data: {:?}", issue_data_lock);
                
                
                                });
                            }
                            None => {},
                        }
                    },
                    _ => {return;},
                }
            },
            
            _ => {},
        }
        self.route = route;
    }
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

    WriteLogger::init(LevelFilter::Info, Config::default(), File::create("rust_cli.log").unwrap()).unwrap();

    // Create a new channel with a capacity of at most 32.
    let (tx, mut rx) = mpsc::channel(8);


    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        // let mut client = client::connect("127.0.0.1:6379").await.unwrap();
    
        // Start receiving messages
        while let Some(cmd) = rx.recv().await {
    
            info!("Manager received Command::{:?}", cmd);
            match cmd {
                Command::LoadLinearTeams { api_key, resp } => {
                    let option_stateful = components::linear_team_select::LinearTeamSelectState::load_teams(api_key).await;
                    info!("LoadLinearTeams data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);

                    // client.get(&key).await;
                },
                Command::LoadLinearIssues { api_key, selected_team, resp } => {
                    // client.set(&key, val).await;
                    let option_stateful = components::linear_issue_display::LinearIssueDisplayState::load_issues(api_key, &selected_team).await;
                    info!("LoadLinearIssuesByTeam data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);
                }
            }
        }
    });


    // Create default app state
    let mut app = App::default();


    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    loop {

        terminal.draw(|mut f| match app.route {
            Route::ActionSelect => {
              ui::draw_action_select(&mut f, &mut app);
            }
            Route::TeamSelect => {
              ui::draw_team_select(&mut f, &mut app);
            }
            Route::LinearInterface => {
                ui::draw_issue_display(&mut f, &mut app);
            }
            _ => {
              panic!()
            }
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Left => {
                    match app.route {
                        Route::ActionSelect => app.actions.unselect(),
                        Route::TeamSelect => {
                            // TODO: Why Doesn't the below line work?
                            // util::state_list::unselect_2(&mut app.linear_team_select_state.teams_state);
                            app.linear_team_select_state.teams_state.select(None);
                        }
                        
                        _ => {}
                        
                    }
                }
                Key::Down => {
                    match app.route {
                        Route::ActionSelect => app.actions.next(),                        
                        Route::TeamSelect => {
                            let handle = &mut *app.linear_team_select_state.teams_data.lock().unwrap();
                            match *handle {
                                Some(ref mut x) => {
                                    // util::state_list::next(&mut app.linear_team_select_state.teams_state, &x.items);
                                    // app.linear_selected_team_idx = app.linear_team_select_state.teams_state.selected();
                                    match x.as_array() {
                                        Some(y) => {
                                            util::state_list::next(&mut app.linear_team_select_state.teams_state, y);
                                            app.linear_selected_team_idx = app.linear_team_select_state.teams_state.selected();
                                        },
                                        None => {},
                                    }
                                }
                                _ => {},
                            }
                        }
                        
                        _ => {}
                    }
                }
                Key::Up => {
                    match app.route {
                        Route::ActionSelect => app.actions.previous(),
                        Route::TeamSelect => {
                            let handle = &mut *app.linear_team_select_state.teams_data.lock().unwrap();
                            match handle {
                                Some(ref mut x) => {
                                    // x.previous();
                                    // app.linear_selected_team_idx = x.state.selected();
                                    match x.as_array() {
                                        Some(y) => {
                                            util::state_list::previous(&mut app.linear_team_select_state.teams_state, y);
                                            app.linear_selected_team_idx = app.linear_team_select_state.teams_state.selected();
                                        },
                                        None => {},
                                    }
                                },
                                _ => {},
                            }
                        }
                        _ => {}
                    }
                }
                Key::Right => {
                    match app.route {
                        Route::ActionSelect => match app.actions.state.selected() {
                            Some(i) => {
                                match i {
                                    0 => { /*app.switch_route(Route::TeamSelect).await*/ app.change_route( Route::TeamSelect, &tx).await }
                                    _ => {}
                                }
                            }
                            _ => {}
                        },
                        // Switch Route as long as a team is selected
                        Route::TeamSelect => match app.linear_selected_team_idx {
                            Some(_) => { app.change_route(Route::LinearInterface, &tx).await },
                            None => {},
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            _ => {

            }
        }
    }

    Ok(())
}