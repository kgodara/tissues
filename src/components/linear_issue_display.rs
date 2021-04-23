use tui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Terminal,
};

use std::sync::{Arc, Mutex};
use serde_json::json;
use serde_json::Value;

// use colorsys::Color as CTColor;

use crate::util::GraphQLCursor;
use crate::linear::client::LinearClient;
use crate::linear::LinearConfig;

use crate::util::ui::{ TableStyle, style_color_from_hex_str };

pub struct LinearIssueDisplay {
    pub issue_table_data: Arc<Mutex<Option<serde_json::Value>>>,
    pub issue_table_state: TableState,
}

impl LinearIssueDisplay {

    pub async fn load_issues(linear_config: LinearConfig, selected_team: &serde_json::Value) -> Option<serde_json::Value> {

        if let serde_json::Value::Object(team) = selected_team {
            let issue_fetch_result = LinearClient::get_issues_by_team(linear_config, 
                                                                        None,
                                                                        selected_team.as_object()
                                                                        .cloned()
                                                                        .unwrap_or(serde_json::Map::default())
                                                                    ).await;

            let mut issues: serde_json::Value = serde_json::Value::Null;
            let mut cursor_info: serde_json::Value = serde_json::Value::Null;

            match issue_fetch_result {
                Ok(x) => { 
                    issues = x["issue_nodes"].clone();
                    cursor_info = x["cursor_info"].clone();
                },
                Err(y) => {
                                info!("Get Issues By Team failed: {:?}", y);
                                return None;
                            },
            }

            info!("Issue Fetch Result: {:?}", issues);

            match issues {
                serde_json::Value::Array(_) => {
                    info!("Populating LinearIssueDisplay::issue_table_data with: {:?}", issues);

                    // return Some(issues);
                    return Some(json!( { "issues": issues, "cursor_info": cursor_info } ));
                },
                _ => {return None;},
            }

        } else {
            return None;
        }
    }

    pub async fn load_issues_paginate(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, selected_team: &serde_json::Value) -> Option<serde_json::Value> {
        if let serde_json::Value::Object(team) = selected_team {
            let issue_fetch_result = LinearClient::get_issues_by_team(linear_config, linear_cursor, selected_team.as_object()
                                                                                    .cloned()
                                                                                    .unwrap_or(serde_json::Map::default())
                                                                    ).await;

            let mut issues: serde_json::Value = serde_json::Value::Null;
            let mut cursor_info: serde_json::Value = serde_json::Value::Null;

            match issue_fetch_result {
                Ok(x) => { 
                    issues = x["issue_nodes"].clone();
                    cursor_info = x["cursor_info"].clone();
                },
                Err(y) => {
                                info!("Get Issues By Team failed: {:?}", y);
                                return None;
                            },
            }

            info!("Issue Fetch Result: {:?}", issues);

            match issues {
                serde_json::Value::Array(_) => {
                    info!("Populating LinearIssueDisplay::issue_table_data with: {:?}", issues);

                    // return Some(issues);
                    return Some(json!( { "issues": issues, "cursor_info": cursor_info } ));
                },
                _ => {return None;},
            }

        } else {
            return None;
        }
    }



    pub fn get_rendered_issue_data(table_data: &Option<serde_json::Value>, table_style: TableStyle) -> Result<Table, &'static str> {

        let table_items;

        let bottom_margin = match table_style.row_bottom_margin {
            Some(margin) => margin,
            None => { 0 }
        };

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
        let header_cells = ["Number", "Title", "State ('m' to modify)", "Description", "createdAt"]
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

                let style_color = style_color_from_hex_str(&color);

                match name {
                    Some(x) => { match style_color {
                        Some(y) => { return Cell::from(x).style(Style::default().fg(y)) },
                        None => return Cell::from(String::default()),
                    }},
                    None => return Cell::from(String::default()),
                }
            };

            cells.insert(2, generate_state_cell());

            Row::new(cells).height(height as u16).bottom_margin(bottom_margin)
        });


        let table_block = Block::default().
                                    borders(Borders::ALL)
                                    .title( match table_style.title_style {
                                        Some(title_style) => {
                
                                            Spans::from(vec![Span::styled(match table_style.view_idx {
                                                                            Some(idx) => vec!["#", idx.to_string().as_str(), " - "].concat(),
                                                                            None => {String::default()}
                                                                            },
                                                                            Style::default()
                                                                ),
                                                                Span::styled(String::from(*title_style.0
                                                                        .as_str()
                                                                        .get_or_insert("Table")
                                                                    ),
                                                                    Style::default()
                                                                        .add_modifier(Modifier::BOLD)
                                                                        .fg(*style_color_from_hex_str(&title_style.1)
                                                                                .get_or_insert(Color::White)
                                                                        )
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
                Constraint::Percentage(20)
            ]);
        
        return Ok(t);

    }





}

impl Default for LinearIssueDisplay {

    fn default() -> LinearIssueDisplay {
        LinearIssueDisplay {
            issue_table_data: Arc::new(Mutex::new(Some(serde_json::Value::Array(vec![])))),
            issue_table_state: TableState::default(),
        }
    }
}
