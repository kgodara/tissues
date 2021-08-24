use serde_json::Value;

use tui::{
    layout::{ Constraint },
    style::{ Style },
    widgets::{ Cell }
};

use crate::util::{
    ui::{ style_color_from_hex_str },
    layout::{ format_str_with_wrap },
};

use crate::constants::table_columns::TableColumn;

pub fn values_to_str(values: &[Value], columns: &[TableColumn]) -> Vec<String> {
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

pub fn format_cell_fields(cell_fields: &[String], widths: &[Constraint], columns: &[TableColumn]) -> Vec<String> {
    cell_fields
        .iter()
        .enumerate()
        .map(|(idx, cell_field)| {
            if let Constraint::Length(width_num) = widths[idx] {
                format_str_with_wrap(cell_field, width_num, columns[idx].max_height)
            } else {
                error!("format_cell_fields - Constraint must be Constraint::Length: {:?}", widths[idx]);
                panic!("format_cell_fields - Constraint must be Constraint::Length: {:?}", widths[idx]);
            }
        })
        .collect()
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