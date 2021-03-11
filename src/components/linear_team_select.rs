use crate::util;
use crate::App;
use crate::graphql;
use crate::linear;

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use std::sync::{ Arc };


pub struct LinearTeamSelectState {
    // serde_json::Value::Array
    pub teams_data: Arc<Option<serde_json::Value>>,
    pub teams_stateful: Option<util::StatefulList<serde_json::Value>>,
}

impl LinearTeamSelectState {

    pub async fn load_teams(&mut self, linear_client: &mut linear::client::LinearClient) {

        info!("Loading teams");

        let team_fetch_result = linear_client.get_teams().await;
        let mut teams: serde_json::Value = serde_json::Value::Null;
      
        match team_fetch_result {
          Ok(x) => { teams = x; }
          Err(y) => {
                        self.teams_data = Arc::new(None);
                        self.teams_stateful = None;
                        return;
                    },
          _ => {}
        }

        // let Ok(teams) = team_fetch_result;

        info!("teams: {}", teams);

        if teams == serde_json::Value::Null {
              // Reset back to previous screen, and continue past loop
              // println!("Team Fetch failed");
              self.teams_data = Arc::new(Some(serde_json::Value::Array(vec![])));
              self.teams_stateful = Some(util::StatefulList::new());
              return;
        }

        let teams_vec;

        match teams.as_array() {
          Some(x) => { teams_vec = x; },
          None => {
            return;
          }
        }

        self.teams_stateful = Some(util::StatefulList::with_items(teams_vec.clone()));
        self.teams_data = Arc::new(Some(teams));
    }





    pub async fn load_teams_2(api_key: Option<String>) -> ( Arc<Option<serde_json::Value>>, Option<util::StatefulList<serde_json::Value>> ) {

        info!("Loading teams");

        let team_fetch_result = linear::client::LinearClient::get_teams_2(api_key).await;
        let mut teams: serde_json::Value = serde_json::Value::Null;
      
        match team_fetch_result {
          Ok(x) => { teams = x; }
          Err(y) => {
                        return (Arc::new(None), None);
                    },
          _ => {}
        }

        // let Ok(teams) = team_fetch_result;

        info!("teams: {}", teams);

        if teams == serde_json::Value::Null {
              // Reset back to previous screen, and continue past loop
              // println!("Team Fetch failed");
              return ( Arc::new(Some(serde_json::Value::Array(vec![]))), Some(util::StatefulList::new()) );
        }

        let teams_vec;

        match teams.as_array() {
          Some(x) => { teams_vec = x; },
          None => {
            return (Arc::new(None), None);
          }
        }

        return ( Arc::new(Some(teams.clone())), Some(util::StatefulList::with_items(teams_vec.clone())));

    }





    pub fn get_rendered_teams_data(teams_stateful: &Option<serde_json::Value>) -> Result<List, &'static str> {

        match teams_stateful {
            Some(input_data) => {

                let starter = input_data.as_array();

                match starter {
                    Some(x) => {
                        let items: Vec<ListItem> = x
                        .iter()
                        .filter_map(|x| {
                            // info!("Filter map on: {:?}", x);
                            if let Some(team_name) = x["name"].as_str() {
                                if let Some(team_key) = x["key"].as_str() {
                                    return Some(format!("{}    {}", team_name, team_key));
                                }
                                else {
                                    return None;
                                }
                            }
                            else {
                                return None;
                            }
                        })
                        .map(|i| {
                            let lines = vec![Spans::from(i)];
                            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
                        })
                        .collect();

    
                        // Create a List from all list items and highlight the currently selected one
                        let items = List::new(items)
                            .block(Block::default().borders(Borders::ALL).title("List"))
                            .highlight_style(
                                Style::default()
                                    .bg(Color::LightGreen)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .highlight_symbol(">> ");
                        return Ok(items);
                    },
                    None => return Err("Failed to convert teams_stateful to vec"),
                }
            }
            // Goal: Err(error) => return Err(error);
            None => {return Err("Stateful List not populated");}
        }

    }

}

impl Default for LinearTeamSelectState {
    fn default() -> LinearTeamSelectState {
        LinearTeamSelectState {
            teams_data: Arc::new(Some(serde_json::Value::Array(vec![]))),
            teams_stateful: Some(util::StatefulList::new()),
        }
    }
}