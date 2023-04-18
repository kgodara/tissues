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


pub fn view_layout(num_views: usize, r: Rect) -> Vec<Rect> {

    // TODO: make this configurable
    let views_per_row: usize = 2;
    let num_rows: usize = (num_views / views_per_row) + (num_views % views_per_row);

    let mut vertical_constraints: Vec<Constraint> = Vec::new();

    for _row in 0..num_rows {
        vertical_constraints.push(Constraint::Percentage(100/(num_rows as u16)));
    }

    let mut row_rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            vertical_constraints,
        )
        .split(r);
    row_rects.reverse();

    let mut final_rects: Vec<Rect> = Vec::new();
    
    let mut rem: usize = num_views;
    for row_idx in 0..num_rows {
        let mut horizontal_constraints: Vec<Constraint> = Vec::new();

        for _col in 0..rem.min(views_per_row) {
            horizontal_constraints.push(Constraint::Percentage(100/(rem.min(views_per_row) as u16)));
        }

        let mut row_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                horizontal_constraints,
            )
            .split(row_rects[row_idx]);
        row_cols.reverse();
        
        final_rects.extend(row_cols);

        rem = rem.saturating_sub(views_per_row);
    }

    final_rects
}