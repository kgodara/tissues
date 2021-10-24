
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

use std::sync::{
    Arc,
    Mutex,
};

use unicode_segmentation::UnicodeSegmentation;

use crate::constants::colors::{ RED, GREEN };

#[derive(Clone, PartialEq)]
pub enum TokenValidationState {
    Null,
    Invalid,
    Validating,
    Valid,
}

pub struct UserInput {
    pub input: String,
    pub token_validation_state: Arc<Mutex<TokenValidationState>>,
}


impl Default for UserInput {
    fn default() -> UserInput {
        UserInput {
            input: String::new(),
            token_validation_state: Arc::new(Mutex::new(TokenValidationState::Null)),
        }
    }
}

impl UserInput {
    pub fn render_help_msg<'a>(input_mode: &'a InputMode, token_validation_state: &TokenValidationState) -> Paragraph<'a> {
        let (mut msg, style) = match input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Press "),
                    // Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    // Span::raw(" to exit, "),
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

        match token_validation_state {
            TokenValidationState::Null => {

            },
            TokenValidationState::Invalid => {
                msg.push(
                    Span::styled(
                        "\nValid Linear Access Token is required.", 
                        Style::default().fg(RED)
                    )
                );
            },
            TokenValidationState::Validating => {
                msg.push(
                    Span::styled(
                        "\nValidating...",
                        Style::default()
                    )
                );
            },
            TokenValidationState::Valid => {
                msg.push(
                    Span::styled(
                        "\nSuccessfully Validated", 
                        Style::default().fg(GREEN)
                    )
                );
            },
        };
    
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text);

        help_message
    }

    pub fn render_input_box<'a>(input: &'a str, input_mode: &InputMode) -> Paragraph<'a> {
        // Generate equivalent amount of '*' chars for each input char
        let grapheme_len: usize = input
            .graphemes(true)
            .count();

        let mut display_str: String = "".to_string();

        for _ in 0..grapheme_len {
            display_str.push('*');
        }

        Paragraph::new(display_str)
            .style(match input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"))
    }
}