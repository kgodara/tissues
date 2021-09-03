use std::fmt::Write;

use serde_json::Value;

use tui::{
    style::{ Color, Modifier, Style },
    widgets::{ Block, Paragraph, Borders, Wrap },
    text::{ Span, Spans },
    layout::{ Alignment }
};

use crate::util::{
    ui::style_color_from_hex_str,
};

// Accepts:
//     issue: JSON object representing issue
// Returns:
//     Paragraph: Paragraph widget containing the colored issue title
pub fn colored_title_from_issue<'a>(issue: Value) -> Paragraph<'a> {

    let issue_color: Value = issue["state"]["color"].clone();
    let issue_number: Value = issue["number"].clone();
    let issue_title: Value = issue["title"].clone();

    // panic! if one of the issue's fields was not found
    if  issue_color.is_null() ||
        issue_number.is_null() ||
        issue_title.is_null()
    {
        error!("colored_title_from_issue: missing required Issue field(s) - color: {:?}, number: {:?}, title: {:?}", issue_color, issue_number, issue_title);
        panic!("colored_title_from_issue: missing required Issue field(s) - color: {:?}, number: {:?}, title: {:?}", issue_color, issue_number, issue_title);
    }

    let mut final_title = String::new();

    write!( &mut final_title,
            "{} - {}",
            issue_number.as_i64().get_or_insert(-0),
            issue_title.as_str().get_or_insert("ERR TITLE NOT FOUND")
    );

    let mut title_color = style_color_from_hex_str(&issue_color);

    let text = vec![
        Spans::from(Span::styled(
            final_title,
            Style::default().fg(*title_color.get_or_insert(Color::Green)).add_modifier(Modifier::ITALIC),
        ))
    ];

    Paragraph::new(text.clone())
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left).wrap(Wrap { trim: true })
}