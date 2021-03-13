use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, List, Table, TableState, ListState},
    Terminal,
};

use std::sync::{Arc, Mutex};

use colorsys::{Rgb};
// use colorsys::Color as CTColor;


use crate::linear::client::LinearClient;
use crate::util::ui::style_color_from_hex_str;

pub struct LinearWorkflowStateDisplayState {
    pub workflow_states_data: Arc<Mutex<Option<serde_json::Value>>>,
    pub workflow_states_state: TableState,
}


impl LinearWorkflowStateDisplayState {

    pub async fn load_workflow_states(api_key: Option<String>) -> Option<serde_json::Value> {

        info!("Loading workflow states");

        let workflow_states_result = LinearClient::get_workflow_states(api_key).await;
        let mut workflow_states: serde_json::Value = serde_json::Value::Null;

        match workflow_states_result {
          Ok(x) => { workflow_states = x; }
          Err(y) => {
                        return None;
                    },
          _ => {}
        }

        if workflow_states == serde_json::Value::Null {
              return Some(serde_json::Value::Array(vec![]));
        }

        match workflow_states {
            serde_json::Value::Array(_) => {
                info!("Populating LinearWorkflowStateDisplayState::workflow_states_data with: {:?}", workflow_states);
                // self.issue_table_data = Arc::new(Mutex::new(Some(issues)));
                return Some(workflow_states);
            },
            _ => {return None;},
        }
    }

    pub fn get_rendered_workflow_state_select(table_data: &Option<serde_json::Value>) -> Result<Table, &'static str> {

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
        let header_cells = ["Name", "Type", "Description"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::LightGreen)));
        
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        let rows = table_array.iter().enumerate().map(|(idx, row)| {

            let cell_fields: std::vec::Vec<std::string::String> = vec![row["type"].clone(), row["description"].clone()]
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
                // let state_obj = row["state"].clone();
                let name = row["name"].clone();
                let color = row["color"].clone();

                let name = match name {
                    serde_json::Value::String(x) => Some(x),
                    _ => None,
                };

                let style_color = style_color_from_hex_str(color);

                match name {
                    Some(x) => { match style_color {
                        Some(y) => { return Cell::from(x).style(Style::default().fg(y)) },
                        None => return Cell::from(String::default()),
                    }},
                    None => return Cell::from(String::default()),
                }
            };

            cells.insert(0, generate_name_cell());

            Row::new(cells).height(height as u16).bottom_margin(1)
        });


        let t = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Select New Workflow State"))
            .highlight_style(selected_style)
            .highlight_symbol(">> ")
            .widths(&[
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(25),
            ]);
        
        return Ok(t);

    }

}



impl Default for LinearWorkflowStateDisplayState {
    fn default() -> LinearWorkflowStateDisplayState {
        LinearWorkflowStateDisplayState {
            workflow_states_data: Arc::new(Mutex::new(Some(serde_json::Value::Array(vec![])))),
            workflow_states_state: TableState::default(),
        }
    }
}