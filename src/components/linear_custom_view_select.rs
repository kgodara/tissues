
use tui::{
    layout::{Constraint},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use std::sync::{ Arc, Mutex };

use serde_json::Value;
use serde_json::json;

use crate::util::GraphQLCursor;
use crate::linear::client::LinearClient;
use crate::linear::LinearConfig;

use crate::util::ui::{ TableStyle, style_color_from_hex_str };



pub struct LinearCustomViewSelect {
    pub view_table_data: Arc<Mutex<Vec<Value>>>,
    pub view_table_state: TableState,
}


impl LinearCustomViewSelect {
    pub async fn load_custom_views(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>) -> Option<Value> {
        let view_fetch_result = LinearClient::get_custom_views(linear_config, linear_cursor).await;

        let views: Value;
        let cursor_info: Value;

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


    pub fn get_rendered_view_data(table_data: &[Value], table_style: TableStyle) -> Result<Table, &'static str> {

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



        let rows = table_data.iter().map(|row| {

            // info!("Table Row Raw: {:?}", row);

            let cell_fields: std::vec::Vec<std::string::String> = vec![row["description"].clone(), row["organization"]["name"].clone(), row["team"]["key"].clone()]
                                .iter()
                                .enumerate()
                                .map(|(i,field)| match field {

                                    serde_json::Value::String(x) => x.clone(),
                                    serde_json::Value::Number(x) => x.clone().as_i64().unwrap_or(0).to_string(),
                                    serde_json::Value::Null => {
                                        // If 'team' is Null, the view is for all teams
                                        if i == 2 {
                                            String::from("All Teams")
                                        }
                                        else {
                                            String::default()
                                        }
                                    },

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
                        Some(y) => { Cell::from(x).style(Style::default().fg(y)) },
                        None => Cell::from(x),
                    }},
                    None => Cell::from(String::default()),
                }
            };

            cells.insert(0, generate_name_cell());

            Row::new(cells).height(height as u16).bottom_margin(1)
        });


        // Determine if this table is selected and should be highlighted by
        // Comparing (table_style.view_idx-1) == (table_style.selected_view_idx)
        let highlight_table = match table_style.view_idx {
            Some(view_idx) => {
                if let Some(selected_view_idx) = table_style.selected_view_idx {
                    view_idx == selected_view_idx
                }
                else { false }
            },
            None => {false}
        };

        let table_block = Block::default()
                                    .borders(Borders::ALL)
                                    .border_style(Style::default().fg(if highlight_table == true { Color::Yellow } else { Color::White }))
                                    .title( match table_style.title_style {
                                        Some(title_style) => {
                
                                            Spans::from(vec![   Span::styled(match table_style.view_idx {
                                                                                Some(idx) => {
                                                                                    vec!["#", idx.to_string().as_str(), " - "].concat()
                                                                                },
                                                                                None => {String::default()}
                                                                            },
                                                                            Style::default()
                                                                ),
                                                                Span::styled(String::from("Select a Custom View"),
                                                                    Style::default()
                                                                        .add_modifier(Modifier::BOLD)
                                                                        .fg(Color::White)
                                                                )
                                                            ])
                                        },
                                        None => { Spans::from(Span::styled("Table", Style::default())) }
                                    });

        let t = Table::new(rows)
            .header(header)
            .block(table_block)
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
            view_table_data: Arc::new(Mutex::new(Vec::new())),
            view_table_state: TableState::default(),
        }
    }
}
