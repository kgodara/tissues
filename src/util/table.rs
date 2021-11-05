use std::cmp::min;

use textwrap::{ wrap };

use serde_json::Value;

use tui::{
    layout::{ Constraint },
    style::{ Color, Modifier, Style },
    text::{Span, Spans},
    widgets::{ Cell }
};

use crate::util::{
    ui::{ style_color_from_hex_str },
    layout::{ format_str_with_wrap },
};

use crate::constants::colors::{ API_REQ_NUM };

use crate::util::loader::{ loader_from_state };

use crate::constants::table_columns::TableColumn;

#[derive(Debug)]
pub struct TableStyle {
    // General (all tables)
    // ( name: Value::String, color_hex_str: Value::String || Value::Null )
    pub title_style: Option<(Value, Value)>,
    pub row_bottom_margin: Option<u16>,

    // View Panel Specific
    pub view_idx: Option<u16>,

    pub highlight_table: bool,

    // Loading State Display Relagted
    pub loading: bool,
    pub loader_state: u16,

    pub req_num: Option<u16>,
}

// Accepts:
//     table_style: style parameters of target table
// Returns:
//     Spans<'a>: a group of Spans generated from provided Style
pub fn gen_table_title_spans<'a>(table_style: TableStyle) -> Spans<'a> {


    match table_style.title_style {
        Some(title_style) => {
                            // Display Table's View index, if provided
            Spans::from(vec![   Span::styled(match table_style.view_idx {
                                                Some(idx) => {
                                                    vec!["#", idx.to_string().as_str(), " - "].concat()
                                                },
                                                None => {String::default()}
                                            },
                                            Style::default()
                                ),
                                // Display Table's Loading State
                                Span::styled(
                                    vec![loader_from_state(table_style.loading, table_style.loader_state).to_string().as_str(), " - "].concat(),
                                    Style::default()
                                ),
                                // Display provided Label as Table Title
                                Span::styled(String::from(*title_style.0
                                        .as_str()
                                        .get_or_insert("Table")
                                    ),
                                    Style::default()
                                        .add_modifier(Modifier::BOLD)
                                        .fg(*style_color_from_hex_str(&title_style.1)
                                                .get_or_insert(Color::White)
                                        )
                                ),
                                // If req # provided, display
                                Span::styled(match table_style.req_num {
                                        Some(req_u16) => { vec![" - Req #: ", req_u16.to_string().as_str()].concat() },
                                        None => { String::default() }
                                    },
                                    Style::default()
                                        .add_modifier(Modifier::ITALIC)
                                        .fg(API_REQ_NUM)
                                )
                            ])
        },
        None => { Spans::from(Span::styled("Table", Style::default())) }
    }
}


pub fn values_to_str_with_fallback(values: &[Value], columns: &[TableColumn]) -> Vec<String> {
    values.iter()
        .enumerate()
        .map(|(idx, field)| match field {

            Value::String(x) => x.clone(),
            Value::Number(x) => x.clone().as_i64().unwrap_or(0).to_string(),
            Value::Null => {
                columns[idx].null_fallback.to_string() 
            },
            _ => {
                String::default()
            },
        })
        .collect()
}

pub fn value_to_str(value: &Value) -> String {
    match value {
        Value::String(x) => x.clone(),
        Value::Number(x) => x.clone().as_i64().unwrap_or(0).to_string(),
        Value::Null => {
            String::default()
        },
        _ => {
            String::default()
        },
    }
}

pub fn format_cell_fields(cell_fields: &[String], widths: &[Constraint], columns: &[TableColumn], col_height: Option<u16>) -> Vec<String> {

    // This should be done at the table-level,
    // so we can determine what the uniform final row height for all rows in the table will be 

    // Get the maximum wrapped-field size that respects max_height (why does it need to respect max height? A: so that height only expands on significant field overflow )
    // e.g. max( .iter().enumerate() min(wrap(cell_fields[idx], widths[idx]), columns[idx].max_height) )
    // pass this to all format_str_with_wrap() calls to enforce all fields take advantage of all available lines

    cell_fields
        .iter()
        .enumerate()
        .map(|(idx, cell_field)| {
            if let Constraint::Length(width_num) = widths[idx] {
                format_str_with_wrap(cell_field,
                    width_num,
                    if let Some(uniform_height) = col_height { uniform_height } else { columns[idx].max_height }
                )
            } else {
                error!("format_cell_fields - Constraint must be Constraint::Length: {:?}", widths[idx]);
                panic!("format_cell_fields - Constraint must be Constraint::Length: {:?}", widths[idx]);
            }
        })
        .collect()
}

// Get the minimum wrapped-field size that respects max_height w.r.t wrapped content height across all fields
// e.g. min( wrap(cell_fields[idx], widths[idx]).len(), columns[idx].max_height )
pub fn row_min_render_height(cell_fields: &[String], widths: &[Constraint], col_defs: &[TableColumn]) -> u16 {
    cell_fields
    .iter()
    .enumerate()
    .map(|(field_idx, _field)| {
        if let Constraint::Length(width_num) = widths[field_idx] {
            min( wrap( &cell_fields[field_idx], width_num as usize).len(), col_defs[field_idx].max_height as usize ) as u16
        } else {
            error!("row_max_render_height() - Constraint must be Constraint::Length: {:?}", widths[field_idx]);
            panic!("row_max_render_height() - Constraint must be Constraint::Length: {:?}", widths[field_idx]);
        }
    })
    .max()
    .unwrap_or(0)
}

pub fn get_row_height(cell_fields: &[String]) -> usize {
    cell_fields
        .iter()
        .map(|content| content.chars().filter(|c| *c == '\n').count())
        .max()
        .unwrap_or(1)
}

pub fn colored_cell(name: String, color: Value) -> Cell<'static> {
    let style_color = style_color_from_hex_str(&color);

    match style_color {
        Some(y) => { Cell::from(name).style(Style::default().fg(y)) },
        None => Cell::from(name),
    }
}