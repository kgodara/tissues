
use std::cmp::max;

use std::sync::{
    Arc,
    Mutex,
    atomic::AtomicBool,
};


use serde_json::Value;

use tui::{
    layout::{ Constraint },
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::linear::view_resolver::ViewLoader;

use crate::util::{
    table::{ values_to_str, format_cell_fields,
        get_row_height, colored_cell,
        TableStyle, gen_table_title_spans
    },
};

use crate::constants::table_columns::{ VIEW_PANEL_COLUMNS };


#[derive(Debug, Clone)]
pub struct DashboardViewPanel {
    pub filter: Value,
    pub issue_table_data: Arc<Mutex<Vec<Value>>>,
    pub view_loader: Arc<Mutex<Option<ViewLoader>>>,
    pub request_num: Arc<Mutex<u32>>,
    pub loading: Arc<AtomicBool>,
}

impl DashboardViewPanel {
    pub fn with_filter(f: Value) -> DashboardViewPanel {
        DashboardViewPanel {
            filter: f,
            issue_table_data: Arc::new(Mutex::new(Vec::new())),
            view_loader: Arc::new(Mutex::new(None)),
            request_num: Arc::new(Mutex::new(0)),
            loading: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn render<'a>(table_data: &[Value], widths: &[Constraint], table_style: TableStyle) -> Result<Table<'a>, &'static str> {

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

        let mut max_seen_row_size: usize = 0;

        let mut rows: Vec<Row> = table_data.iter()
            .map(|row| {

                let cell_fields: Vec<String> = 
                    values_to_str(
                        &[row["number"].clone(),
                            row["title"].clone(),
                            row["state"]["name"].clone(),
                            row["description"].clone(),
                            row["createdAt"].clone()
                        ],
                        &VIEW_PANEL_COLUMNS
                    );

                // Get the formatted Strings for each cell field
                let cell_fields_formatted: Vec<String> = format_cell_fields(&cell_fields, widths, &VIEW_PANEL_COLUMNS);

                max_seen_row_size = max(get_row_height(&cell_fields_formatted), max_seen_row_size);

                let mut cells: Vec<Cell> = cell_fields_formatted.iter().map(|c| Cell::from(c.clone())).collect();

                let name = cell_fields_formatted[2].clone();
                let color = row["state"]["color"].clone();

                // Insert new "state" cell, and remove unformatted version
                cells.insert(2, colored_cell(name, color));
                cells.remove(3);

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


        let table_block = Block::default()
                                    .borders(Borders::ALL)
                                    .border_style(Style::default().fg(if table_style.highlight_table { Color::Yellow } else { Color::White }))
                                    .title( gen_table_title_spans(table_style) );

        let t = Table::new(rows)
            .header(header)
            .block(table_block)
            .highlight_style(selected_style);

        Ok(t)

    }
}





impl Default for DashboardViewPanel {

    fn default() -> DashboardViewPanel {
        DashboardViewPanel {
            filter: Value::Null,
            issue_table_data: Arc::new(Mutex::new(Vec::new())),
            view_loader: Arc::new(Mutex::new(None)),
            request_num: Arc::new(Mutex::new(0)),
            loading: Arc::new(AtomicBool::new(false)),
        }
    }
}
