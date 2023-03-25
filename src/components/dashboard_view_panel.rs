
use std::cmp::{ max };

use std::sync::{
    Arc,
    Mutex,
    atomic::AtomicBool,
};

use tui::{
    layout::{ Constraint },
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};


use crate::linear::{
    schema::{ Issue, CustomView }
};

use crate::util::{
    table::{ empty_str_to_fallback, format_cell_fields,
        row_min_render_height, get_row_height, colored_cell,
        TableStyle, gen_table_title_spans
    },
    GraphQLCursor,
};

use crate::constants::table_columns::{ VIEW_PANEL_COLUMNS };


#[derive(Debug, Clone)]
pub struct DashboardViewPanel {
    pub view: CustomView,
    pub issue_table_data: Arc<Mutex<Vec<Issue>>>,
    pub view_cursor: Arc<Mutex<Option<GraphQLCursor>>>,
    pub loading: Arc<AtomicBool>,
}

impl DashboardViewPanel {
    pub fn with_view(f: CustomView) -> DashboardViewPanel {
        DashboardViewPanel {
            view: f,
            issue_table_data: Arc::new(Mutex::new(Vec::new())),
            view_cursor: Arc::new(Mutex::new(None)),
            loading: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn render<'a>(table_data: &[Issue], widths: &[Constraint], table_style: TableStyle) -> Result<Table<'a>, &'static str> {

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

        // Get the maximum wrapped-field size that respects max_height across all rows this will be the final uniform row height
        // e.g. max( .iter().enumerate() min( wrap(cell_fields[idx], widths[idx]).len(), columns[idx].max_height ) )
        // pass this to all format_str_with_wrap() calls to enforce all fields take advantage of all available lines

        let mut cell_fields_list: Vec<Vec<String>> = Vec::new();

        let max_row_size_opt: Option<u16> = table_data
            .iter()
            .map(|issue| {

                let cell_fields: Vec<String> = 
                    empty_str_to_fallback(
                        &[  &issue.number.to_string().clone(),
                            &issue.title.clone(),
                            &issue.state.name.clone(),
                            issue.description.as_deref().unwrap_or(""),
                            &issue.created_at.clone()
                        ],
                        &VIEW_PANEL_COLUMNS
                    );

                cell_fields_list.push(cell_fields.clone());

                row_min_render_height(&cell_fields, widths, &VIEW_PANEL_COLUMNS)
            })
            .max();

        let mut rows: Vec<Row> = table_data.iter()
            .enumerate()
            .map(|(idx, issue)| {

                // Get the formatted Strings for each cell field
                let cell_fields_formatted: Vec<String> = format_cell_fields(&cell_fields_list[idx], widths, &VIEW_PANEL_COLUMNS, max_row_size_opt);

                max_seen_row_size = max(get_row_height(&cell_fields_formatted), max_seen_row_size);

                let mut cells: Vec<Cell> = cell_fields_formatted.iter().map(|c| Cell::from(c.clone())).collect();

                let name = cell_fields_formatted[2].clone();

                // Insert new "state" cell, and remove unformatted version
                cells.insert(2, colored_cell(name, &issue.state.color.clone()));
                cells.remove(3);

                Row::new(cells)
                    .bottom_margin(bottom_margin)
            })
            .collect();

        // Set all row heights to max_seen_row_size
        rows = rows.into_iter()
            .map(|row| {
                // row.height(max_seen_row_size as u16)
                if let Some(x) = max_row_size_opt {
                    row.height(x)
                } else {
                    row.height(max_seen_row_size as u16)
                }
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
            view: CustomView::default(),
            issue_table_data: Arc::new(Mutex::new(Vec::new())),
            view_cursor: Arc::new(Mutex::new(None)),
            loading: Arc::new(AtomicBool::new(false)),
        }
    }
}
