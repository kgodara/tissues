use crate::app::InputMode;

use std::{error::Error, io};

use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use std::sync::{
    Arc,
    Mutex,
};

use unicode_segmentation::UnicodeSegmentation;

use crate::constants::colors::{ RED, GREEN };

#[derive(Clone, PartialEq)]
pub enum InputContext {
    Token,
    IssueTitle,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TokenValidationState {
    Null,
    Invalid,
    Validating,
    Valid,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TitleValidationState {
    Null,
    Valid,
    Invalid,
}
#[derive(Clone, Copy, PartialEq)]
pub enum ValidationState {
    Token(TokenValidationState),
    Title(TitleValidationState),
}


pub struct UserInput {
    pub input: String,
    pub cursor_offset: usize,

    pub token_validation_state: Arc<Mutex<TokenValidationState>>,
    pub title_validation_state: TitleValidationState,
}


impl Default for UserInput {
    fn default() -> UserInput {
        UserInput {
            input: String::new(),
            cursor_offset: 0,

            token_validation_state: Arc::new(Mutex::new(TokenValidationState::Null)),
            title_validation_state: TitleValidationState::Null,
        }
    }
}

impl UserInput {

    pub fn gen_status_msg(input_context: InputContext, validation_state: ValidationState) -> Option<Span<'static>> {
        match input_context {
            InputContext::Token => {
                if let ValidationState::Token(token_validation_state) = validation_state {
                    match token_validation_state {
                        TokenValidationState::Null => { None },
                        TokenValidationState::Invalid => {
                            Some(Span::styled(
                                "\nInvalid Linear Access Token", 
                                Style::default().fg(RED)
                            ))
                        },
                        TokenValidationState::Validating => {
                            Some(Span::styled(
                                "\nValidating...",
                                Style::default()
                            ))
                        },
                        TokenValidationState::Valid => {
                            Some(Span::styled(
                                "\nValidated", 
                                Style::default().fg(GREEN)
                            ))
                        },
                    }
                } else {
                    None
                }
            },
            InputContext::IssueTitle => {
                if let ValidationState::Title(title_validation_state) = validation_state {
                    match title_validation_state {
                        TitleValidationState::Null => { None },
                        TitleValidationState::Valid => { 
                            Some(Span::styled(
                                "\nValid",
                                Style::default().fg(GREEN)
                            ))
                        },
                        TitleValidationState::Invalid => { 
                            Some(Span::styled(
                                "\nInvalid", 
                                Style::default().fg(RED)
                            ))
                        },
                    }
                } else {
                    None
                }
            },
        }
    }

    pub fn render_help_msg<'a>(input_mode: &'a InputMode, input_context: InputContext, validation_state: &ValidationState) -> Paragraph<'a> {
        let (mut msg, style) = match input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Press "),
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
                    match input_context {
                        InputContext::Token => { Span::raw(" to submit token") },
                        InputContext::IssueTitle => { Span::raw(" to submit title") }
                    }
                ],
                Style::default(),
            ),
        };

        if let Some(status_span) = UserInput::gen_status_msg(input_context, *validation_state) {
            msg.push(status_span);
        }
    
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text);

        help_message
    }

    pub fn render_input_box<'a>(input: &'a str, input_context: InputContext, input_mode: &InputMode) -> Paragraph<'a> {
        // Generate equivalent amount of '*' chars for each input char
        let grapheme_len: usize = input
            .graphemes(true)
            .count();

        let mut display_str: String = "".to_string();

        match input_context {
            InputContext::Token => {   
                for _ in 0..grapheme_len {
                    display_str.push('*');
                }
            },
            InputContext::IssueTitle => { 
                display_str = input.to_string();
            }
        }

        Paragraph::new(display_str)
            .style(match input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"))
    }

    pub fn unicode_width(&self) -> usize {
        unicode_width::UnicodeWidthStr::width(self.input.as_str())
    }

    pub fn set_input(&mut self, input_init: String) {
        self.input = input_init;
        // set initial cursor pos to end of input str
        self.cursor_offset = self.unicode_width() + 1;
        debug!("set_input() - cursor_offset: {}", self.cursor_offset);
    }

    pub fn with_input(input_init: String) -> UserInput {
        UserInput {
            input: input_init.clone(),
            cursor_offset: unicode_width::UnicodeWidthStr::width(input_init.as_str())+1,

            token_validation_state: Arc::new(Mutex::new(TokenValidationState::Null)),
            title_validation_state: TitleValidationState::Null,
        }
    }

    pub fn insert(&mut self, ch: char) {
        let gr_inds = self.input.grapheme_indices(true)
              .collect::<Vec<(usize, &str)>>();

        // debug!("user_input::insert() - self.input: {:?}", self.input);
        // debug!("user_input::insert() - gr_ind: {:?}", gr_inds);
        // debug!("user_input::insert() - idx: {:?}", self.cursor_offset);

        let insert_byte_idx: usize;

        // if pushing to end of self.input use push
        if (self.cursor_offset-1) == gr_inds.len() {
            self.input.push(ch);
        } else {
            insert_byte_idx = gr_inds[self.cursor_offset-1].0;
            drop(gr_inds);
            self.input.insert(insert_byte_idx, ch);
        }

        self.cursor_offset += 1;
    }

    pub fn delete(&mut self) {
        let gr_inds = self.input.grapheme_indices(true)
            .collect::<Vec<(usize, &str)>>();

        // do nothing if at start of line
        if self.cursor_offset < 2 {
            return;
        } else if (self.cursor_offset-1) == gr_inds.len() {
            // pop if at end of line
            self.input.pop();
        } else {
            let delete_byte_idx: usize = gr_inds[self.cursor_offset-2].0;
            drop(gr_inds);
            self.input.remove(delete_byte_idx);
        }
        self.cursor_offset -= 1;
    }

    pub fn move_cursor_back(&mut self) {
        if self.cursor_offset > 1 { self.cursor_offset -= 1; }
        debug!("move_cursor_back() - cursor_offset: {}", self.cursor_offset);
    }

    pub fn move_cursor_forwards(&mut self) {
        if self.cursor_offset <= self.unicode_width() { self.cursor_offset += 1; }
    }
}