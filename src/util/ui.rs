use tui::{
    style::{Color},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
};
use colorsys::{Rgb};

use serde_json::Value;

#[derive(Debug)]
pub struct TableStyle {
    // ( name: Value::String, color_hex_str: Value::String || Value::Null )
    pub title_style: Option<(Value, Value)>,
    pub view_idx: Option<u16>,
    pub selected_view_idx: Option<u16>,
    pub row_bottom_margin: Option<u16>,
    pub req_num: Option<u16>,
}

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

pub fn style_color_from_hex_str(color: &serde_json::Value) -> Option<Color> {

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


    return Some(Color::Rgb(rgb_struct.red() as u8, rgb_struct.green() as u8, rgb_struct.blue() as u8));

}

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
