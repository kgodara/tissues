use tui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::util::ui::{style_color_from_hex_str, TableStyle, gen_table_title_spans};

use crate::constants::table_columns::{ DASHBOARD_VIEW_CONFIG_COLUMNS };
use crate::util::layout::{ format_str_with_wrap };


use serde_json::Value;

pub struct DashboardViewDisplay {
    pub view_table_state: TableState,
}

impl DashboardViewDisplay {

    pub fn get_rendered_view_table<'a>(view_list: &'a [Option<Value>],
        widths: &Vec<Constraint>,
        table_style: TableStyle,
        bbox: &Rect) -> Result<Table<'a>, &'static str> {

        let bottom_margin = table_style.row_bottom_margin.unwrap_or(0);

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::DarkGray);
        let header_cells: Vec<Cell> = DASHBOARD_VIEW_CONFIG_COLUMNS
            .iter()
            .map(|h| Cell::from(&*h.label).style(Style::default().fg(Color::LightGreen)))
            .collect();

        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        // info!("Header: {:?}", header);



        let rows = view_list.iter().enumerate().map(|(idx, row_option)| {

            // Get the String representations of each cell field

            let cell_fields: Vec<String> = match row_option {

                Some(row) => {
                    vec![row["name"].clone(), row["description"].clone(), row["organization"]["name"].clone(), row["team"]["key"].clone()]
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
                                .collect()
                },
                None => vec![String::default(), String::default(), String::default()],
            };

            // Get the formatted Strings for each cell field
            let cell_fields_formatted: Vec<String> = cell_fields.iter()
                .enumerate()
                .map(|(idx, cell_field)| {
                    if let Constraint::Length(width_num) = widths[idx] {
                        format_str_with_wrap(cell_field, width_num, DASHBOARD_VIEW_CONFIG_COLUMNS[idx].max_height)
                    } else {
                        error!("get_rendered_view_table - Constraint must be Constraint::Length: {:?}", widths[idx]);
                        panic!("get_rendered_view_table - Constraint must be Constraint::Length: {:?}", widths[idx]);
                    }
                })
                .collect();

            debug!("get_rendered_view_table - cell_fields_formatted: {:?}", cell_fields_formatted);

            let height = cell_fields_formatted
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;

            debug!("get_rendered_view_table - height: {:?}", height);

            let mut cells: Vec<Cell> = cell_fields_formatted.iter().map(|c| Cell::from(c.clone())).collect();

            let generate_name_cell = || {
                match row_option {
                    Some(row) => {
                        let name: String = cell_fields_formatted[0].clone();
                        let color = row["color"].clone();

                        /*
                        let name = match name {
                            Value::String(x) => Some(x),
                            _ => None,
                        };
                        */

                        let style_color = style_color_from_hex_str(&color);

                        match style_color {
                            Some(y) => { Cell::from(name).style(Style::default().fg(y)) },
                            None => Cell::from(name),
                        }
                        /*
                        match name {
                            Some(x) => { match style_color {
                                Some(y) => { Cell::from(x).style(Style::default().fg(y)) },
                                None => Cell::from(x),
                            }},
                            None => Cell::from(String::default()),
                        }
                        */
                    },
                    None => { Cell::from(String::from("Empty Slot"))}
                }
            };

            // Insert new "name" cell, and remove unformatted version
            cells.insert(0, generate_name_cell());
            cells.remove(1);

            Row::new(cells).height(height as u16).bottom_margin(bottom_margin)
        });


        // Get widths based on TableColumns

        // lazy_static! provides a struct which dereferences towards target struct, hence: '&*'
        // https://github.com/rust-lang-nursery/lazy-static.rs/issues/119#issuecomment-419595818
        // debug!("get_rendered_view_table - widths_from_rect(): {:?}", widths_from_rect(bbox, &*DASHBOARD_VIEW_CONFIG_COLUMNS));

        // let widths: Vec<Constraint> = widths_from_rect(bbox, &*DASHBOARD_VIEW_CONFIG_COLUMNS);

        let t = Table::new(rows)
            .header(header)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if table_style.highlight_table { Color::Yellow } else { Color::White }))
                .title( gen_table_title_spans(table_style) )
            )
            .highlight_style(selected_style);

        Ok(t)

    }
}

impl Default for DashboardViewDisplay {

    fn default() -> DashboardViewDisplay {
        DashboardViewDisplay {
            view_table_state: TableState::default(),
        }
    }
}
