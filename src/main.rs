#![allow(dead_code)]

use std::io;
use serde_json;

mod graphql;
mod linear;
mod ui;
mod util;

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



fn get_platform() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    println!("Buffer: {}", buffer);
    Ok(buffer)
}

enum Route {
    ActionSelect,
    TeamSelect,
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
    // Available actions
    actions: util::StatefulList<&'a str>,
}

impl<'a> Default for App<'a> {
    fn default() -> App<'a> {
        App {
            route: Route::ActionSelect,
            input: String::new(),
            linear_client: linear::client::LinearClient::default(),
            actions: util::StatefulList::with_items(vec![
                "Create Issue",
                "Test",
            ]),
        }
    }
}




fn main() -> Result<(), Box<dyn std::error::Error>> {


    // Create default app state
    let mut app = App::default();


    /*
    let mut issue_variables = serde_json::Map::new();

    issue_variables.insert(String::from("title"), serde_json::Value::String(String::from("Test Rust-CLI 1")));
    issue_variables.insert(String::from("description"), serde_json::Value::String(String::from("Made From Rust")));
    issue_variables.insert(String::from("teamId"), serde_json::Value::String(String::from("3e2c3a3a-c883-432f-9877-dcbb8785650a")));


    let mutation_response;
    mutation_response = create_linear_issue(&contents, issue_variables);

    match mutation_response {
        Ok(mutation_response) => {println!("Mutation Success: {}", mutation_response)},
        Err(mutation_response) => {println!("Mutation Failed: {}", mutation_response)},
    }
    */

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
                    app.actions.unselect();
                }
                Key::Down => {
                    app.actions.next();
                }
                Key::Up => {
                    app.actions.previous();
                }
                Key::Right => {
                    app.route = Route::TeamSelect
                }
                _ => {}
            },
            _ => {

            }
        }
      

        /*
        terminal.draw(|f| {


            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(15),
                        Constraint::Percentage(60),
                        Constraint::Percentage(5),
                        Constraint::Percentage(10)
                    ].as_ref()
                )
                .split(f.size());
            let block = Block::default()
                .title("Block")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[0]);
            let block = Block::default()
                .title("Block 2")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[2]);
            let block = Block::default()
                .title("Block 3")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[4]);
        }).expect("Didn't work");
        */
    }

    Ok(())



}