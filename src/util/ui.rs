use tui::{
    style::{Color, Modifier, Style},
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans}
};

use crate::util::colors::{ API_REQ_NUM };

use crate::util::loader::{ loader_from_state };

use colorsys::{Rgb};

use serde_json::Value;

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


// Useful for modals
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}


// Coloring
pub fn style_color_from_hex_str(color: &Value) -> Option<Color> {

    let hex_str;

    match color {
        serde_json::Value::String(x) => hex_str = x,
        _ => { return None }
    };

    let rgb_struct;

    match Rgb::from_hex_str(hex_str.as_str()).ok() {
        Some(x) => rgb_struct = x,
        None => return None,
    }


    Some(Color::Rgb(rgb_struct.red() as u8, rgb_struct.green() as u8, rgb_struct.blue() as u8))

}

pub fn hex_str_from_style_color(color: &Color) -> Option<String> {

    match color {
        Color::Rgb(r, g, b) => {
            // TODO: Fix this temp workaround
            let rgb = Rgb::new(*r as f64, *g as f64, *b as f64, None);
            Some(rgb.to_hex_string())
        },
        _ => None
    }
}

// View Panel Arrangements
pub fn single_view_layout(idx: usize, r: Rect) -> Rect {
    if idx != 0 {
        panic!("idx must be 0 for single view layout, requested {:?}", idx);
    }
    r
}

pub fn double_view_layout(idx: usize, r: Rect) -> Rect {

    if idx > 1 {
        panic!("double_view_layout invalid idx: {:?}", idx);
    }

    let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Percentage(100)
        ]
        .as_ref(),
    )
    .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(popup_layout[0])[idx]
}

pub fn three_view_layout(idx: usize, r: Rect) -> Rect {
    if idx > 2 {
        panic!("three_view_layout invalid idx: {:?}", idx);
    }

    let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]
        .as_ref(),
    )
    .split(r);

    if idx == 0 {
        return Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(100),
                ]
                .as_ref(),
            )
            .split(popup_layout[0])[0];
    }

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[idx-1]
}

pub fn four_view_layout(idx: usize, r: Rect) -> Rect {
    if idx > 3 {
        panic!("four_view_layout invalid idx: {:?}", idx);
    }

    let vertical_idx = if idx < 2 { 0 } else { 1 };
    let horizontal_idx = if idx % 2 == 0 { 0 } else { 1 };

    let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]
        .as_ref(),
    )
    .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(popup_layout[vertical_idx])[horizontal_idx]
}

pub fn five_view_layout(idx: usize, r: Rect) -> Rect {
    if idx > 4 {
        panic!("five_view_layout invalid idx: {:?}", idx);
    }

    let mut vertical_idx = 0;
    let mut horizontal_idx = 0;

    if idx > 0 {
        vertical_idx = if (idx-1) < 2 { 0 } else { 1 };
        horizontal_idx = if (idx-1) % 2 == 0 { 0 } else { 1 };
    }

    let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ]
        .as_ref(),
    )
    .split(r);

    if idx == 0 {
        return Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(100),
                    ]
                    .as_ref(),
                )
                .split(popup_layout[0])[0]
    }

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(popup_layout[vertical_idx+1])[horizontal_idx]
    
}

pub fn six_view_layout(idx: usize, r: Rect) -> Rect {
    if idx > 5 {
        panic!("six_view_layout invalid idx: {:?}", idx);
    }

    let vertical_idx = if idx < 2 { 0 } else if idx < 4 { 1 } else { 2 };
    let horizontal_idx = if idx % 2 == 0 { 0 } else { 1 };

    let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ]
        .as_ref(),
    )
    .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(popup_layout[vertical_idx])[horizontal_idx]
}
