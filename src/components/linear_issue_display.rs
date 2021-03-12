use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Terminal,
};

use std::sync::{Arc, Mutex};

use colorsys::{Rgb};
// use colorsys::Color as CTColor;


use crate::linear::client::LinearClient;

pub struct LinearIssueDisplayState {
    pub issue_table_data: Arc<Mutex<Option<serde_json::Value>>>,
    pub issue_table_state: TableState,
}

impl LinearIssueDisplayState {
    pub async fn load_issues(&mut self, linear_client: &LinearClient, selected_team: &serde_json::Value) {

        if let serde_json::Value::Object(team) = selected_team {
            let issue_fetch_result = linear_client.get_issues_by_team(selected_team.as_object()
                                                                                    .cloned()
                                                                                    .unwrap_or(serde_json::Map::default())
                                                                    ).await;

            let mut issues: serde_json::Value = serde_json::Value::Null;

            match issue_fetch_result {
                Ok(x) => { issues = x; },
                Err(y) => {
                                info!("Get Issues By Team failed: {:?}", y);
                                self.issue_table_data = Arc::new(Mutex::new(None));
                                return;
                            },
            }

            info!("Issue Fetch Result: {:?}", issues);

            match issues {
                serde_json::Value::Array(_) => {
                    info!("Populating LinearIssueDisplayState::issue_table_data with: {:?}", issues);
                    self.issue_table_data = Arc::new(Mutex::new(Some(issues)));
                },
                _ => {return;},
            }

        } else {
            return;
        }
    }

    pub async fn load_issues_2(api_key: Option<String>, selected_team: &serde_json::Value) -> Option<serde_json::Value> {

        if let serde_json::Value::Object(team) = selected_team {
            let issue_fetch_result = LinearClient::get_issues_by_team_2(api_key, selected_team.as_object()
                                                                                    .cloned()
                                                                                    .unwrap_or(serde_json::Map::default())
                                                                    ).await;

            let mut issues: serde_json::Value = serde_json::Value::Null;

            match issue_fetch_result {
                Ok(x) => { issues = x; },
                Err(y) => {
                                info!("Get Issues By Team failed: {:?}", y);
                                return None;
                            },
            }

            info!("Issue Fetch Result: {:?}", issues);

            match issues {
                serde_json::Value::Array(_) => {
                    info!("Populating LinearIssueDisplayState::issue_table_data with: {:?}", issues);
                    // self.issue_table_data = Arc::new(Mutex::new(Some(issues)));
                    return Some(issues);
                },
                _ => {return None;},
            }

        } else {
            return None;
        }
    }

    pub fn style_color_from_hex_str(color: serde_json::Value) -> Option<Color> {

        let hex_str;

        match color {
            serde_json::Value::String(x) => hex_str = x,
            _ => { return None }
        };

        let rgb_struct;

        match Rgb::from_hex_str(hex_str.as_str()).ok() {
            Some(x) => rgb_struct = x,
            None => return None,
        }


        return Some(Color::Rgb(rgb_struct.red() as u8, rgb_struct.green() as u8, rgb_struct.blue() as u8));

    }



    pub fn get_rendered_issue_data(table_data: &Option<serde_json::Value>) -> Result<Table, &'static str> {

        let table_items;

        match table_data {
            Some(x) => table_items = x,
            None => { return Err("Table Items is None"); }
        }

        let table_array;
        match table_items.as_array() {
            Some(x) => table_array = x,
            None => { return Err("table_data is not an Array") }
        }

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::DarkGray);
        let header_cells = ["Number", "Title", "State", "Description", "createdAt"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::LightGreen)));
        
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        // info!("Header: {:?}", header);



        let rows = table_array.iter().enumerate().map(|(idx, row)| {

            // info!("Table Row Raw: {:?}", row);

            let cell_fields: std::vec::Vec<std::string::String> = vec![row["number"].clone(), row["title"].clone(), row["description"].clone(), row["createdAt"].clone()]
                                .iter()
                                .map(|field| match field {

                                    serde_json::Value::String(x) => x.clone(),
                                    serde_json::Value::Number(x) => x.clone().as_i64().unwrap_or(0).to_string(),
                                    serde_json::Value::Null => String::default(),
                                    
                                    _ => { String::default() },
                                })
                                .collect();



            // info!("Cell Fields: {:?}", cell_fields);

            let height = cell_fields
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;

            // info!("Height: {:?}", height);

            let mut cells: Vec<Cell> = cell_fields.iter().map(|c| Cell::from(c.clone())).collect();

            let generate_state_cell = || {
                let state_obj = row["state"].clone();
                let name = state_obj["name"].clone();
                let color = state_obj["color"].clone();

                let name = match name {
                    serde_json::Value::String(x) => Some(x),
                    _ => None,
                };

                let style_color = super::linear_issue_display::LinearIssueDisplayState::style_color_from_hex_str(color);

                match name {
                    Some(x) => { match style_color {
                        Some(y) => { return Cell::from(x).style(Style::default().fg(y)) },
                        None => return Cell::from(String::default()),
                    }},
                    None => return Cell::from(String::default()),
                }
            };

            cells.insert(2, generate_state_cell());

            Row::new(cells).height(height as u16).bottom_margin(1)
        });


        let t = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Table"))
            .highlight_style(selected_style)
            .highlight_symbol(">> ")
            .widths(&[
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
                Constraint::Percentage(20)
            ]);
        
        return Ok(t);

    }





}

impl Default for LinearIssueDisplayState {

    fn default() -> LinearIssueDisplayState {
        LinearIssueDisplayState {
            issue_table_data: Arc::new(Mutex::new(Some(serde_json::Value::Array(vec![])))),
            issue_table_state: TableState::default(),
        }
    }
}
