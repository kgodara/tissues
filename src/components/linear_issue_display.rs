use tui::{
    layout::{Constraint },
    style::{Color, Modifier, Style},
    text::{Span, Spans },
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use std::sync::{Arc, Mutex};
use serde_json::json;
use serde_json::Value;

// use colorsys::Color as CTColor;

use crate::util::ui::{ TableStyle, style_color_from_hex_str, gen_table_title_spans };
use crate::util::colors::{ API_REQ_NUM };
use crate::constants::table_columns::{ VIEW_PANEL_COLUMNS };

pub struct LinearIssueDisplay {
    pub issue_table_data: Arc<Mutex<Option<serde_json::Value>>>,
    pub issue_table_state: TableState,
}

impl LinearIssueDisplay {

    pub fn get_rendered_issue_data(table_data: &[Value], table_style: TableStyle) -> Result<Table, &'static str> {

        let bottom_margin = table_style.row_bottom_margin.unwrap_or(0);

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::DarkGray);

        let header_cells: Vec<Cell> = VIEW_PANEL_COLUMNS
            .iter()
            .map(|h| Cell::from(&*h.label).style(Style::default().fg(Color::LightGreen)))
            .collect();

        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        // info!("Header: {:?}", header);



        let rows = table_data.iter().map(|row| {

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
                        Some(y) => { Cell::from(x).style(Style::default().fg(y)) },
                        None => Cell::from(String::default()),
                    }},
                    None => Cell::from(String::default()),
                }
            };

            cells.insert(2, generate_state_cell());

            Row::new(cells).height(height as u16).bottom_margin(bottom_margin)
        });

        let table_block = Block::default()
                                    .borders(Borders::ALL)
                                    .border_style(Style::default().fg(if table_style.highlight_table { Color::Yellow } else { Color::White }))
                                    .title( gen_table_title_spans(table_style) );

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
