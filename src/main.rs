#![allow(dead_code)]
#[allow(unused_imports)]

#[macro_use]
extern crate lazy_static;


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
mod network;

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

use command::{ Command,
                get_cmd,
                exec_add_cmd,
                exec_replace_cmd,
                exec_delete_cmd,
                exec_select_view_panel_cmd,
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

    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        // let mut client = client::connect("127.0.0.1:6379").await.unwrap();
    
        // Start receiving messages
        while let Some(cmd) = rx.recv().await {

            info!("Manager received IOEvent::{:?}", cmd);
            match cmd {
                IOEvent::LoadCustomViews { linear_config, linear_cursor, resp } => {
                    let option_stateful = components::linear_custom_view_select::LinearCustomViewSelect::load_custom_views(linear_config, Some(linear_cursor)).await;
                    info!("LoadCustomViews data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);
                },
                IOEvent::LoadViewIssues { linear_config, view, view_loader, resp } => {
                    // let option_stateful = linear::view_resolver::get_issues_from_view(&view, linear_config).await;
                    let issue_list = linear::view_resolver::optimized_view_issue_fetch(&view, view_loader, linear_config).await;
                    info!("LoadViewIssues data: {:?}", issue_list);

                    let _ = resp.send(issue_list);
                },
                IOEvent::LoadLinearTeams { api_key, resp } => {
                    let option_stateful = components::linear_team_select::LinearTeamSelectState::load_teams(api_key).await;
                    info!("LoadLinearTeams data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);

                    // client.get(&key).await;
                },
                IOEvent::LoadLinearIssues { linear_config, selected_team, resp } => {
                    // client.set(&key, val).await;
                    let option_stateful = components::linear_issue_display::LinearIssueDisplay::load_issues(linear_config, &selected_team).await;
                    info!("LoadLinearIssuesByTeam data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);
                },
                IOEvent::LoadLinearIssuesPaginate { linear_config, linear_cursor, selected_team, resp } => {
                    let option_stateful = components::linear_issue_display::LinearIssueDisplay::load_issues_paginate(linear_config, Some(linear_cursor), &selected_team).await;
                    info!("LoadLinearIssuesPaginate data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);
                },
                IOEvent::LoadWorkflowStates { api_key, selected_team, resp } => {
                    let option_stateful = components::linear_workflow_state_display::LinearWorkflowStateDisplayState::load_workflow_states_by_team(api_key, &selected_team).await;
                    info!("LoadWorkflowStates data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);
                },
                IOEvent::UpdateIssueWorkflowState { api_key, selected_issue, selected_workflow_state, resp } => {

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

    terminal.clear()?;

    let mut tick_idx = 0u64;
    let mut cmd_option: Option<Command>;

    loop {

        terminal.draw(|mut f| match app.route {
            Route::ActionSelect => {
              ui::draw_action_select(&mut f, &mut app);
            },
            Route::DashboardViewDisplay => {
                ui::draw_dashboard_view_display(&mut f, &mut app);
            },
            Route::CustomViewSelect => {
                ui::draw_view_select(&mut f, &mut app);
            },
            Route::TeamSelect => {
              ui::draw_team_select(&mut f, &mut app);
            }
            Route::LinearInterface => {
                ui::draw_issue_display(&mut f, &mut app);
            }
        })?;
        let event_next = events.next()?;

        match event_next {
            Event::Input(input) => {
                // Update Command String / Get Command to apply
                cmd_option = get_cmd(&mut app.cmd_str, input);
                info!("cmd_option: {:?}", cmd_option);

                if let Some(cmd) = cmd_option {
                    tick_idx = 0;
                    app.cmd_str.clear();

                    // Execute Command
                    match cmd {
                        Command::Quit => {
                            break;
                        },
                        Command::Add => {
                            exec_add_cmd(&mut app, &tx).await;
                        },
                        Command::Replace => {
                            exec_replace_cmd(&mut app, &tx).await;
                        },
                        Command::Delete => {
                            exec_delete_cmd(&mut app, &tx).await;
                        },
                        Command::SelectViewPanel(idx) => {
                            // linear_dashboard_view_panel_selected
                            exec_select_view_panel_cmd(&mut app, idx, &tx).await;
                        }
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
                tick_idx += 1;
            },
        };
    }

    Ok(())
}