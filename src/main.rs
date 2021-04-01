#![allow(dead_code)]

use std::io;
use std::fs;

use serde_json;

mod app;
mod graphql;
mod linear;
mod ui;
mod util;
mod errors;
mod command;

mod components;

use app::Route as Route;
use app::Platform as Platform;

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

    // Create a new channel with a capacity of at most 8.
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
                Command::LoadLinearIssues { linear_config, selected_team, resp } => {
                    // client.set(&key, val).await;
                    let option_stateful = components::linear_issue_display::LinearIssueDisplayState::load_issues(linear_config, &selected_team).await;
                    info!("LoadLinearIssuesByTeam data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);
                },
                Command::LoadWorkflowStates { api_key, selected_team, resp } => {
                    let option_stateful = components::linear_workflow_state_display::LinearWorkflowStateDisplayState::load_workflow_states_by_team(api_key, &selected_team).await;
                    info!("LoadWorkflowStates data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);
                },
                Command::UpdateIssueWorkflowState { api_key, selected_issue, selected_workflow_state, resp } => {

                    // Get id field from issue Object
                    let issue_id = selected_issue["id"].clone();
                    // Get id field from workflow state Object
                    let workflow_state_id = selected_workflow_state["id"].clone();

                    // Return if id not found for the issue or workflow state
                    if issue_id == serde_json::Value::Null || workflow_state_id == serde_json::Value::Null {
                        let _ = resp.send(None);
                    }
                    else {
                        let mut issue_update_variables = serde_json::Map::new();

                        issue_update_variables.insert(String::from("issueId"), issue_id);
                        issue_update_variables.insert(String::from("newStateId"), workflow_state_id);

                        let option_stateful = linear::client::LinearClient::update_issue_workflow_state(api_key, issue_update_variables).await;

                        let _ = resp.send(option_stateful.ok());
                    }
                }
            }
        }
    });


    // Create default app state
    let mut app = app::App::default();


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
                // Quit program
                Key::Char('q') => {
                    break;
                },
                // Only relevant when user is in position to modify an issue's workflow state
                Key::Char('m') => {
                    match app.route {
                        // Create pop-up on top of issue display component
                        Route::LinearInterface => {
                            // Dispatch event to begin loading new data
                            app.dispatch_event("load_workflows", &tx);

                            // Enable drawing of workflow state selection pop-up
                            app.set_draw_issue_state_select(Platform::Linear, true);
                        }
                        _ => {},
                    }
                }
                Key::Left => {
                    match app.route {

                        // Unselect from List of Actions
                        Route::ActionSelect => app.actions.unselect(),

                        // Unselect from Selection of Teams
                        Route::TeamSelect => {
                            util::state_list::unselect(&mut app.linear_team_select.teams_state);
                        },

                        // Unselect from list of Linear Issues
                        Route::LinearInterface => {
                            util::state_table::unselect(&mut app.linear_issue_display.issue_table_state);
                        }

                        _ => {}
                        
                    }
                }
                Key::Down => {
                    match app.route {
                        // Select next Action
                        Route::ActionSelect => app.actions.next(),

                        // Select next team from list of Linear teams and update 'app.linear_selected_team_idx'
                        Route::TeamSelect => {
                            let handle = &mut *app.linear_team_select.teams_data.lock().unwrap();
                            match *handle {
                                Some(ref mut x) => {
                                    match x.as_array() {
                                        Some(y) => {
                                            util::state_list::next(&mut app.linear_team_select.teams_state, y);
                                            app.linear_selected_team_idx = app.linear_team_select.teams_state.selected();
                                        },
                                        None => {},
                                    }
                                }
                                _ => {},
                            }
                        },

                        // Select next issue from list of Linear issues and update 'app.linear_selected_issue_idx'
                        Route::LinearInterface => {
                            // If User is not selecting a new workflow state for an issue, select next issue
                            if *app.draw_issue_state_select(Platform::Linear) == false {
                                let handle = &mut *app.linear_issue_display.issue_table_data.lock().unwrap();
                                match *handle {
                                    Some(ref mut x) => {
                                        match x.as_array() {
                                            Some(y) => {
                                                util::state_table::next(&mut app.linear_issue_display.issue_table_state, y);
                                                app.linear_selected_issue_idx = app.linear_issue_display.issue_table_state.selected();
                                                info!("app.linear_selected_issue_idx: {:?}", app.linear_selected_issue_idx);
                                            },
                                            None => {},
                                        }
                                    }
                                    _ => {},
                                }
                            }
                            // If User is selecting a new workflow state for an issue, select next workflow state
                            else {
                                info!("Attempting to scroll down on Workflow State Selection");
                                let handle = &mut *app.linear_workflow_select.workflow_states_data.lock().unwrap();
                                match *handle {
                                    Some(ref mut x) => {
                                        match x.as_array() {
                                            Some(y) => {
                                                util::state_table::next(&mut app.linear_workflow_select.workflow_states_state, y);
                                                app.linear_selected_workflow_state_idx = app.linear_workflow_select.workflow_states_state.selected();
                                                // info!("app.linear_selected_workflow_state_idx: {:?}", app.linear_selected_workflow_state_idx);
                                            },
                                            None => {},
                                        }
                                    },
                                    None => {}
                                }
                            }
                        }
                        
                        _ => {}
                    }
                }
                Key::Up => {
                    match app.route {
                        Route::ActionSelect => app.actions.previous(),
                        Route::TeamSelect => {
                            let handle = &mut *app.linear_team_select.teams_data.lock().unwrap();
                            match handle {
                                Some(ref mut x) => {
                                    match x.as_array() {
                                        Some(y) => {
                                            util::state_list::previous(&mut app.linear_team_select.teams_state, y);
                                            app.linear_selected_team_idx = app.linear_team_select.teams_state.selected();
                                        },
                                        None => {},
                                    }
                                },
                                _ => {},
                            }
                        },
                        Route::LinearInterface => {
                            // If User is not selecting a new workflow state for an issue, select previous issue
                            if *app.draw_issue_state_select(Platform::Linear) == false {
                                let handle = &mut *app.linear_issue_display.issue_table_data.lock().unwrap();
                                match *handle {
                                    Some(ref mut x) => {
                                        match x.as_array() {
                                            Some(y) => {
                                                util::state_table::previous(&mut app.linear_issue_display.issue_table_state, y);
                                                app.linear_selected_issue_idx = app.linear_issue_display.issue_table_state.selected();
                                                info!("app.linear_selected_issue_idx: {:?}", app.linear_selected_issue_idx);
                                            },
                                            None => {},
                                        }
                                    }
                                    _ => {},
                                }
                            }
                            // If User is selecting a new workflow state for an issue, select previous workflow state
                            else {
                                info!("Attempting to scroll up on Workflow State Selection");
                                let handle = &mut *app.linear_workflow_select.workflow_states_data.lock().unwrap();
                                match *handle {
                                    Some(ref mut x) => {
                                        match x.as_array() {
                                            Some(y) => {
                                                util::state_table::previous(&mut app.linear_workflow_select.workflow_states_state, y);
                                                app.linear_selected_workflow_state_idx = app.linear_workflow_select.workflow_states_state.selected();
                                            },
                                            None => {},
                                        }
                                    },
                                    None => {}
                                }
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
                                    0 => { app.change_route( Route::TeamSelect, &tx).await }
                                    _ => {}
                                }
                            }
                            _ => {}
                        },
                        // Switch Route as long as a team is selected
                        Route::TeamSelect => match app.linear_selected_team_idx {
                            Some(_) => { app.change_route(Route::LinearInterface, &tx).await },
                            None => {},
                        },
                        // Dispatch Update Issue Workflow State command if User selects a workflow state for a given Issue
                        Route::LinearInterface => {
                            app.dispatch_event("update_issue_workflow", &tx);
                            // Close Workflow States Panel
                            app.set_draw_issue_state_select(Platform::Linear, false);

                        },
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