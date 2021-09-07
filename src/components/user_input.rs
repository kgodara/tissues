
use crate::app::InputMode;

use crate::util::event::{Event, Events};
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use crate::constants::colors::{ RED };

pub struct UserInput {
    pub input: String,
    pub access_token_not_set: bool,
    pub invalid_access_token_len: bool,
}

impl Default for UserInput {
    fn default() -> UserInput {
        UserInput {
            input: String::new(),
            access_token_not_set: false,
            invalid_access_token_len: false,
        }
    }
}

impl UserInput {
    pub fn render_help_msg(input_mode: &InputMode, access_token_not_set: bool, invalid_access_token_len: bool) -> Paragraph{
        let (mut msg, style) = match input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to start editing."),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    Span::raw("Press "),
                    Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to stop editing, "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to record the message"),
                ],
                Style::default(),
            ),
        };

        if access_token_not_set {
            msg.push(
                Span::styled(
                    "\nA Linear Access Token is required.", 
                    Style::default().fg(RED)
                )
            );
        } else if invalid_access_token_len {
            msg.push(
                Span::styled(
                    "\nInvalid Linear Access Token Length.", 
                    Style::default().fg(RED)
                )
            );
        }
    
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text);

        return help_message;
    }

    pub fn render_input_box<'a>(input: &'a str, input_mode: &InputMode) -> Paragraph<'a> {
        Paragraph::new(input.as_ref())
            .style(match input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"))
    }
}