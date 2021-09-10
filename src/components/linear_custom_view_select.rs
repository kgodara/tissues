use std::cmp::max;

use tui::{
    layout::{Constraint},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use std::sync::{
    Arc,
    Mutex, 
    atomic::{ AtomicBool }
};

use serde_json::{ Value, json};

use crate::linear::{
    client::LinearClient,
    LinearConfig
};

use crate::util::{
    table::{ values_to_str_with_fallback, format_cell_fields,
        get_row_height, colored_cell,
        TableStyle, gen_table_title_spans
    },
    GraphQLCursor
};

use crate::constants::table_columns::{ CUSTOM_VIEW_SELECT_COLUMNS };


pub struct LinearCustomViewSelect {
    pub view_table_data: Arc<Mutex<Vec<Value>>>,
    pub view_table_state: TableState,
    pub loading: Arc<AtomicBool>,
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
            Value::Array(_) => {
                info!("Populating LinearCustomViewSelect::view_table_data with: {:?}", views);

                // return Some(issues);
                return Some(json!( { "views": views, "cursor_info": cursor_info } ));
            },
            _ => {return None;},
        }
    }


    pub fn render<'a>(table_data: &[Value],
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

        let mut max_seen_row_size: usize = 0;

        let mut rows: Vec<Row> = table_data.iter()
            .map(|row| {

                let cell_fields: Vec<String> = values_to_str_with_fallback(
                    &[row["name"].clone(),
                        row["description"].clone(),
                        row["organization"]["name"].clone(),
                        row["team"]["key"].clone()
                    ],
                    &CUSTOM_VIEW_SELECT_COLUMNS
                );

                // Get the formatted Strings for each cell field
                let cell_fields_formatted: Vec<String> = format_cell_fields(&cell_fields, widths, &CUSTOM_VIEW_SELECT_COLUMNS);

                // debug!("get_rendered_view_data - cell_fields_formatted: {:?}", cell_fields_formatted);

                max_seen_row_size = max(get_row_height(&cell_fields_formatted), max_seen_row_size);

                let mut cells: Vec<Cell> = cell_fields_formatted.iter().map(|c| Cell::from(c.clone())).collect();

                let name: String = cell_fields_formatted[0].clone();
                let color = row["color"].clone();

                // Insert new "name" cell, and remove unformatted version
                cells.insert(0, colored_cell(name, color));
                cells.remove(1);

                // debug!("render - row: {:?}", Row::new(cells.clone()).bottom_margin(bottom_margin));

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




impl Default for LinearCustomViewSelect {

    fn default() -> LinearCustomViewSelect {
        LinearCustomViewSelect {
            view_table_data: Arc::new(Mutex::new(Vec::new())),
            view_table_state: TableState::default(),
            loading: Arc::new(AtomicBool::new(false)),
        }
    }
}
