
use tui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Terminal,
};

use std::sync::{ Arc, Mutex };

use serde_json::Value;
use serde_json::json;

use crate::util::GraphQLCursor;
use crate::linear::client::LinearClient;
use crate::linear::LinearConfig;

use crate::util::ui::style_color_from_hex_str;



pub struct LinearCustomViewSelect {
    pub view_table_data: Arc<Mutex<Option<Value>>>,
    pub view_table_state: TableState,
}


impl LinearCustomViewSelect {
    pub async fn load_custom_views(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>) -> Option<Value> {
        let view_fetch_result = LinearClient::get_custom_views(linear_config, linear_cursor).await;

        let mut views: serde_json::Value = serde_json::Value::Null;
        let mut cursor_info: serde_json::Value = serde_json::Value::Null;

        match view_fetch_result {
            Ok(x) => { 
                views = x["view_nodes"].clone();
                cursor_info = x["cursor_info"].clone();
            },
            Err(y) => {
                info!("Get Custom Views failed: {:?}", y);
                return None;
            },
        }

        info!("Custom View Fetch Result: {:?}", views);

        match views {
            serde_json::Value::Array(_) => {
                info!("Populating LinearCustomViewSelect::view_table_data with: {:?}", views);

                // return Some(issues);
                return Some(json!( { "views": views, "cursor_info": cursor_info } ));
            },
            _ => {return None;},
        }
    }


    pub fn get_rendered_view_data(table_data: &Option<serde_json::Value>) -> Result<Table, &'static str> {

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
        let header_cells = ["Name", "Description", "Organization", "Team"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::LightGreen)));
        
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        // info!("Header: {:?}", header);



        let rows = table_array.iter().enumerate().map(|(idx, row)| {

            // info!("Table Row Raw: {:?}", row);

            let cell_fields: std::vec::Vec<std::string::String> = vec![row["description"].clone(), row["organization"]["name"].clone(), row["team"]["key"].clone()]
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

            let generate_name_cell = || {
                let name = row["name"].clone();
                let color = row["color"].clone();

                let name = match name {
                    serde_json::Value::String(x) => Some(x),
                    _ => None,
                };

                let style_color = style_color_from_hex_str(&color);

                match name {
                    Some(x) => { match style_color {
                        Some(y) => { return Cell::from(x).style(Style::default().fg(y)) },
                        None => return Cell::from(x),
                    }},
                    None => return Cell::from(String::default()),
                }
            };

            cells.insert(0, generate_name_cell());

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
                Constraint::Percentage(25),
                Constraint::Percentage(20),
            ]);
        
        return Ok(t);

    }
}




impl Default for LinearCustomViewSelect {

    fn default() -> LinearCustomViewSelect {
        LinearCustomViewSelect {
            view_table_data: Arc::new(Mutex::new(None)),
            view_table_state: TableState::default(),
        }
    }
}
