use tui::{
    style::{Color},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
};
use colorsys::{Rgb};

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