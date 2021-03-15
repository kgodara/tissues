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

    // Selected Linear Issue
    linear_selected_issue_idx: Option<usize>,

    // Linear Workflow Select State
    linear_workflow_select_state: components::linear_workflow_state_display::LinearWorkflowStateDisplayState,

    // Selected Linear Workflow State
    linear_selected_workflow_state_idx: Option<usize>,

    // Draw Workflow State Selection panel
    linear_draw_workflow_state_select: bool,


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
            linear_selected_issue_idx: None,
            
            linear_workflow_select_state: components::linear_workflow_state_display::LinearWorkflowStateDisplayState::default(),
            linear_selected_workflow_state_idx: None,

            linear_draw_workflow_state_select: false,

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

    fn dispatch_event(&mut self, event_name: &str, tx: &tokio::sync::mpsc::Sender<Command>) {

        match event_name {
            "load_workflows" => {
                let tx2 = tx.clone();

                let api_key = self.linear_client.config.api_key.clone();

                let workflow_data_handle = self.linear_workflow_select_state.workflow_states_data.clone();


                let t1 = tokio::spawn(async move {

                    let (resp_tx, resp_rx) = oneshot::channel();

                    let cmd = Command::LoadWorkflowStates { api_key: api_key, resp: resp_tx };
                    tx2.send(cmd).await.unwrap();

                    let res = resp_rx.await.ok();

                    info!("LoadWorkflowStates Command returned: {:?}", res);

                    let mut workflow_data_lock = workflow_data_handle.lock().unwrap();

                    match res {
                        Some(x) => {
                            *workflow_data_lock = x;
                        }
                        None => {},
                    }

                    info!("New self.linear_workflow_select_state.workflow_states_data: {:?}", workflow_data_lock);
                });
            },
            "update_issue_workflow" => {
                let tx3 = tx.clone();

                let api_key = self.linear_client.config.api_key.clone();

                // Need to get selected Workflow State and selected Issue
                let issue_data_handle = self.linear_issue_display_state.issue_table_data.clone();
                let workflow_state_data_handle = self.linear_workflow_select_state.workflow_states_data.clone();

                // Get Linear selected Issue index
                match self.linear_selected_issue_idx {
                    Some(issue_idx) => {
                        // Acquire a lock on Linear Issue data
                        match &*issue_data_handle.lock().unwrap() {
                            Some(issue_data) => {
                                // Get Linear selected Workflow State index
                                match self.linear_selected_workflow_state_idx {
                                    Some(workflow_idx) => {
                                        // Acquire a lock on Linear Workflow state data
                                        match &*workflow_state_data_handle.lock().unwrap() {
                                            Some(workflow_data) => {
                                                // Acquire relevant issue and workflow state
                                                let selected_issue = issue_data[issue_idx].clone();
                                                let selected_workflow_state = workflow_data[workflow_idx].clone();

                                                // Spawn task to issue command to update workflow state
                                                let t3 = tokio::spawn( async move {
                                                    let (resp2_tx, resp2_rx) = oneshot::channel();

                                                    let cmd = Command::UpdateIssueWorkflowState {   api_key: api_key,
                                                                                                    selected_issue: selected_issue,
                                                                                                    selected_workflow_state: selected_workflow_state,
                                                                                                    resp: resp2_tx  
                                                                                                };
                                                    tx3.send(cmd).await.unwrap();

                                                    let res = resp2_rx.await.ok();

                                                    info!("UpdateIssueWorkflowState Command returned: {:?}", res);

                                                    // Match existing Issue in display and update with returned issue

                                                    /*
                                                    let mut workflow_data_lock = workflow_data_handle.lock().unwrap();
                                
                                                    match res {
                                                        Some(x) => {
                                                            *workflow_data_lock = x;
                                                        }
                                                        None => {},
                                                    }
                                                    */

                                                });
                                            },
                                            None => {},
                                        }

                                    },
                                    None => {}
                                };
                            },
                            None => {},
                        };
                    },
                    None => {},
                }
            }
            _ => {return},
        }

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
                Command::LoadLinearIssues { api_key, selected_team, resp } => {
                    // client.set(&key, val).await;
                    let option_stateful = components::linear_issue_display::LinearIssueDisplayState::load_issues(api_key, &selected_team).await;
                    info!("LoadLinearIssuesByTeam data: {:?}", option_stateful);

                    let _ = resp.send(option_stateful);
                },
                Command::LoadWorkflowStates { api_key, resp } => {
                    let option_stateful = components::linear_workflow_state_display::LinearWorkflowStateDisplayState::load_workflow_states(api_key).await;
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
                            app.linear_draw_workflow_state_select = true;
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
                            util::state_list::unselect(&mut app.linear_team_select_state.teams_state);
                        },

                        // Unselect from list of Linear Issues
                        Route::LinearInterface => {
                            util::state_table::unselect(&mut app.linear_issue_display_state.issue_table_state);
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
                            let handle = &mut *app.linear_team_select_state.teams_data.lock().unwrap();
                            match *handle {
                                Some(ref mut x) => {
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
                        },

                        // Select next issue from list of Linear issues and update 'app.linear_selected_issue_idx'
                        Route::LinearInterface => {
                            // If User is not selecting a new workflow state for an issue, select next issue
                            if app.linear_draw_workflow_state_select == false {
                                let handle = &mut *app.linear_issue_display_state.issue_table_data.lock().unwrap();
                                match *handle {
                                    Some(ref mut x) => {
                                        match x.as_array() {
                                            Some(y) => {
                                                util::state_table::next(&mut app.linear_issue_display_state.issue_table_state, y);
                                                app.linear_selected_issue_idx = app.linear_issue_display_state.issue_table_state.selected();
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
                                let handle = &mut *app.linear_workflow_select_state.workflow_states_data.lock().unwrap();
                                match *handle {
                                    Some(ref mut x) => {
                                        match x.as_array() {
                                            Some(y) => {
                                                util::state_table::next(&mut app.linear_workflow_select_state.workflow_states_state, y);
                                                app.linear_selected_workflow_state_idx = app.linear_workflow_select_state.workflow_states_state.selected();
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
                            let handle = &mut *app.linear_team_select_state.teams_data.lock().unwrap();
                            match handle {
                                Some(ref mut x) => {
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
                        },
                        Route::LinearInterface => {
                            // If User is not selecting a new workflow state for an issue, select previous issue
                            if app.linear_draw_workflow_state_select == false {
                                let handle = &mut *app.linear_issue_display_state.issue_table_data.lock().unwrap();
                                match *handle {
                                    Some(ref mut x) => {
                                        match x.as_array() {
                                            Some(y) => {
                                                util::state_table::previous(&mut app.linear_issue_display_state.issue_table_state, y);
                                                app.linear_selected_issue_idx = app.linear_issue_display_state.issue_table_state.selected();
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
                                let handle = &mut *app.linear_workflow_select_state.workflow_states_data.lock().unwrap();
                                match *handle {
                                    Some(ref mut x) => {
                                        match x.as_array() {
                                            Some(y) => {
                                                util::state_table::previous(&mut app.linear_workflow_select_state.workflow_states_state, y);
                                                app.linear_selected_workflow_state_idx = app.linear_workflow_select_state.workflow_states_state.selected();
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
                        },
                        // Dispatch Update Issue Workflow State command if User selects a workflow state for a given Issue
                        Route::LinearInterface => {
                            app.dispatch_event("update_issue_workflow", &tx);
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