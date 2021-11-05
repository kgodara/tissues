
use std::cmp::max;

use serde_json::Value;

use tui::{
    layout::{Constraint},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::util::{
    table::{ TableStyle, gen_table_title_spans, 
        values_to_str_with_fallback, format_cell_fields,
        get_row_height, colored_cell, row_min_render_height
    },
};

use crate::constants::table_columns::{ DASHBOARD_VIEW_CONFIG_COLUMNS };


pub struct DashboardViewConfigDisplay {
    pub view_table_state: TableState,
}

impl DashboardViewConfigDisplay {

    pub fn render<'a>(view_list: &'a [Option<Value>],
        widths: &[Constraint],
        table_style: TableStyle,
    ) -> Result<Table<'a>, &'static str> {

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

        let mut max_seen_row_size: usize = 0;

        let mut cell_fields_list: Vec<Vec<String>> = Vec::new();

        let max_row_size_opt: Option<u16> = view_list
            .iter()
            .map(|row_option| {

                // Get the String representations of each cell field
                let cell_fields: Vec<String> = match row_option {
                    Some(row) => {
                        values_to_str_with_fallback(
                            &[row["name"].clone(),
                                row["description"].clone(),
                                row["organization"]["name"].clone(),
                                row["team"]["key"].clone()
                            ],
                            &DASHBOARD_VIEW_CONFIG_COLUMNS
                        )
                    },
                    None => vec![String::default(), String::default(), String::default()],
                };

                cell_fields_list.push(cell_fields.clone());

                row_min_render_height(&cell_fields, widths, &DASHBOARD_VIEW_CONFIG_COLUMNS)
            })
            .max();

        let mut rows: Vec<Row> = view_list.iter()
            .enumerate()
            .map(|(idx, row_option)| {

                // Get the formatted Strings for each cell field
                let cell_fields_formatted: Vec<String> = format_cell_fields(&cell_fields_list[idx], widths, &DASHBOARD_VIEW_CONFIG_COLUMNS, max_row_size_opt);

                // debug!("get_rendered_view_table - cell_fields_formatted: {:?}", cell_fields_formatted);
                
                max_seen_row_size = max(get_row_height(&cell_fields_formatted), max_seen_row_size);

                let mut cells: Vec<Cell> = cell_fields_formatted.iter().map(|c| Cell::from(c.clone())).collect();

                let generate_name_cell = || {
                    match row_option {
                        Some(row) => {
                            let name: String = cell_fields_formatted[0].clone();
                            let color = row["color"].clone();

                            colored_cell(name, color)
                        },
                        None => { Cell::from(String::from("Empty Slot"))}
                    }
                };

                // Insert new "name" cell, and remove unformatted version
                cells.insert(0, generate_name_cell());
                cells.remove(1);

                Row::new(cells)
                    .bottom_margin(bottom_margin)
            })
            .collect();
        // Set all row heights to max_seen_row_size
        rows = rows.into_iter()
        .map(|row| {
            row.height(max_seen_row_size as u16)
        })
        .collect();

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

impl Default for DashboardViewConfigDisplay {

    fn default() -> DashboardViewConfigDisplay {
        DashboardViewConfigDisplay {
            view_table_state: TableState::default(),
        }
    }
}