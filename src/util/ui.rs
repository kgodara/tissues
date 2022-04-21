use tui::{
    style::{ Color },
    layout::{Constraint, Direction, Layout, Rect},
};

use colorsys::{Rgb};



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

pub fn style_color_from_hex_str(color: &str) -> Option<Color> {

    let rgb_struct = match Rgb::from_hex_str(color).ok() {
        Some(x) => x,
        None => return None,
    };


    Some(Color::Rgb(rgb_struct.red() as u8, rgb_struct.green() as u8, rgb_struct.blue() as u8))

}



pub fn hex_str_from_style_color(color: &Color) -> Option<String> {

    match color {
        Color::Rgb(r, g, b) => {
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
