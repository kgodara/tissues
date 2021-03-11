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
    async fn switch_route(&mut self, route: Route) {
        match route {
            Route::ActionSelect => {},
            Route::TeamSelect => {
                self.linear_team_select_state.load_teams(&mut self.linear_client).await;
            },
            Route::LinearInterface => {
                // Some team is selected
                match self.linear_selected_team_idx {
                    Some(idx) => {
                        // Arc<Option<T>> -> Option<&T>
                        match &*self.linear_team_select_state.teams_data {
                            Some(data) => self.linear_issue_display_state.load_issues(&self.linear_client, &data[idx]).await,
                            None => {},
                        }
                    },
                    _ => {return;},
                }
            },
        }
        self.route = route;
    }


    async fn change_route(&mut self, route: Route, tx: &tokio::sync::mpsc::Sender<Command>) {
        match route {
            Route::ActionSelect => {},
            Route::TeamSelect => {

                let tx2 = tx.clone();

                let api_key = self.linear_client.config.api_key.clone();


                let t1 = tokio::spawn(async move {
                    let cmd = Command::LoadLinearTeams { api_key: api_key };
                    tx2.send(cmd).await.unwrap();
                });


                // self.linear_team_select_state.load_teams(&mut self.linear_client).await;
            },
            Route::LinearInterface => {
                // Some team is selected
                match self.linear_selected_team_idx {
                    Some(idx) => {
                        // Arc<Option<T>> -> Option<&T>
                        match &*self.linear_team_select_state.teams_data {
                            Some(data) => self.linear_issue_display_state.load_issues(&self.linear_client, &data[idx]).await,
                            None => {},
                        }
                    },
                    _ => {return;},
                }
            },
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
                Command::LoadLinearTeams { api_key } => {
                    let ( data, stateful ) = components::linear_team_select::LinearTeamSelectState::load_teams_2(api_key).await;
                    info!("LoadLinearTeams data: {:?}", data);
                    // client.get(&key).await;
                },
                Command::LoadLinearIssues { selected_team } => {
                    // client.set(&key, val).await;
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
                        Route::TeamSelect => match  app.linear_team_select_state.teams_stateful {
                            Some(ref mut x) => x.unselect(),
                            _ => {},
                        }
                        _ => {}
                    }
                }
                Key::Down => {
                    match app.route {
                        Route::ActionSelect => app.actions.next(),
                        Route::TeamSelect => match  app.linear_team_select_state.teams_stateful {
                            Some(ref mut x) => {
                                x.next();
                                app.linear_selected_team_idx = x.state.selected();
                            }
                            _ => {},
                        }
                        _ => {}
                    }
                }
                Key::Up => {
                    match app.route {
                        Route::ActionSelect => app.actions.previous(),
                        Route::TeamSelect => match  app.linear_team_select_state.teams_stateful {
                            Some(ref mut x) => {
                                x.previous();
                                app.linear_selected_team_idx = x.state.selected();
                            },
                            _ => {},
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
                            Some(_) => { app.switch_route(Route::LinearInterface).await },
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