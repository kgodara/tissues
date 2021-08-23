
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

use crate::util::ui::{ TableStyle, style_color_from_hex_str, gen_table_title_spans };

use crate::constants::table_columns::{ CUSTOM_VIEW_SELECT_COLUMNS };

use crate::util::layout::{ format_str_with_wrap };


pub struct LinearCustomViewSelect {
    pub view_table_data: Arc<Mutex<Vec<Value>>>,
    pub view_table_state: TableState,
    pub loading: Arc<Mutex<bool>>,
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


    pub fn get_rendered_view_data<'a>(table_data: &[Value],
        widths: &[Constraint],
        table_style: TableStyle) -> Result<Table<'a>, &'static str> {

        let bottom_margin = table_style.row_bottom_margin.unwrap_or(0);

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::DarkGray);
        let header_cells: Vec<Cell> = CUSTOM_VIEW_SELECT_COLUMNS
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

            let cell_fields: Vec<String> = vec![row["name"].clone(), row["description"].clone(), row["organization"]["name"].clone(), row["team"]["key"].clone()]
                                .iter()
                                .enumerate()
                                .map(|(i,field)| match field {

                                    Value::String(x) => x.clone(),
                                    Value::Number(x) => x.clone().as_i64().unwrap_or(0).to_string(),
                                    Value::Null => {
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

            // Get the formatted Strings for each cell field
            let cell_fields_formatted: Vec<String> = cell_fields.iter()
                .enumerate()
                .map(|(idx, cell_field)| {
                    if let Constraint::Length(width_num) = widths[idx] {
                        format_str_with_wrap(cell_field, width_num, CUSTOM_VIEW_SELECT_COLUMNS[idx].max_height)
                    } else {
                        error!("get_rendered_view_data - Constraint must be Constraint::Length: {:?}", widths[idx]);
                        panic!("get_rendered_view_data - Constraint must be Constraint::Length: {:?}", widths[idx]);
                    }
                })
                .collect();
            
            debug!("get_rendered_view_data - cell_fields_formatted: {:?}", cell_fields_formatted);

            // info!("Cell Fields: {:?}", cell_fields);

            let height = cell_fields_formatted
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;

            // info!("Height: {:?}", height);

            let mut cells: Vec<Cell> = cell_fields_formatted.iter().map(|c| Cell::from(c.clone())).collect();

            let generate_name_cell = || {
                let name: String = cell_fields_formatted[0].clone();
                let color = row["color"].clone();

                let style_color = style_color_from_hex_str(&color);

                match style_color {
                    Some(y) => { Cell::from(name).style(Style::default().fg(y)) },
                    None => Cell::from(name),
                }
            };

            // Insert new "name" cell, and remove unformatted version
            cells.insert(0, generate_name_cell());
            cells.remove(1);


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
            .widths(&[
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(25),
                Constraint::Percentage(20),
            ]);
        
        Ok(t)

    }
}




impl Default for LinearCustomViewSelect {

    fn default() -> LinearCustomViewSelect {
        LinearCustomViewSelect {
            view_table_data: Arc::new(Mutex::new(Vec::new())),
            view_table_state: TableState::default(),
            loading: Arc::new(Mutex::new(false)),
        }
    }
}
