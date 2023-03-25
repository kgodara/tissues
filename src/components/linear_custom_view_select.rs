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

use crate::linear::{
    schema::CustomView,
};

use crate::util::{
    table::{ empty_str_to_fallback, format_cell_fields,
        row_min_render_height, get_row_height, colored_cell,
        TableStyle, gen_table_title_spans
    },
};

use crate::constants::table_columns::{ CUSTOM_VIEW_SELECT_COLUMNS };


pub struct LinearCustomViewSelect {
    pub view_table_data: Arc<Mutex<Vec<CustomView>>>,
    pub view_table_state: TableState,
    pub loading: Arc<AtomicBool>,
}


impl LinearCustomViewSelect {

    pub fn render<'a>(table_data: &[CustomView],
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


        // Get the maximum wrapped-field size that respects max_height across all rows this will be the final uniform row height
        // e.g. max( .iter().enumerate() min( wrap(cell_fields[idx], widths[idx]).len(), columns[idx].max_height ) )
        // pass this to all format_str_with_wrap() calls to enforce all fields take advantage of all available lines

        let mut cell_fields_list: Vec<Vec<String>> = Vec::new();

        let max_row_size_opt: Option<u16> = table_data
            .iter()
            .map(|custom_view| {

                let cell_fields: Vec<String> = empty_str_to_fallback(
                    &[
                        &custom_view.name,
                        &custom_view.description.clone().unwrap_or(String::from("")),
                        &custom_view.organization.name,
                        match &custom_view.team {
                            Some(team) => { &team.key },
                            None => { "" },
                        }
                    ],
                    &CUSTOM_VIEW_SELECT_COLUMNS
                );

                cell_fields_list.push(cell_fields.clone());

                row_min_render_height(&cell_fields, widths, &CUSTOM_VIEW_SELECT_COLUMNS)
            })
            .max();


        let mut rows: Vec<Row> = table_data.iter()
            .enumerate()
            .map(|(idx, custom_view)| {

                // Get the formatted Strings for each cell field
                let cell_fields_formatted: Vec<String> = format_cell_fields(&cell_fields_list[idx], widths, &CUSTOM_VIEW_SELECT_COLUMNS, max_row_size_opt);

                // debug!("get_rendered_view_data - cell_fields_formatted: {:?}", cell_fields_formatted);

                max_seen_row_size = max(get_row_height(&cell_fields_formatted), max_seen_row_size);

                let mut cells: Vec<Cell> = cell_fields_formatted.iter().map(|c| Cell::from(c.clone())).collect();

                let name: String = cell_fields_formatted[0].clone();

                // Insert new "name" cell, and remove unformatted version
                cells.insert(0, colored_cell(name, &custom_view.color.clone().unwrap_or("#000000".to_string())));
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
