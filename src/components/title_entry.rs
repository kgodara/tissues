

use tui::{
    backend::Backend,
    layout::{ Constraint, Direction, Rect, Layout },
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Paragraph, Block, Borders },
    Frame,
};

use std::sync::{ Arc, Mutex };

use crate::components::user_input::UserInput;

use crate::constants::colors::{ RED, GREEN };

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TitleValidationState {
    Null,
    Invalid,
    Validating,
    Valid,
}


#[derive(Debug)]
pub struct TitleEntry {
    pub input: UserInput,
    pub title_validation_state: Arc<Mutex<TitleValidationState>>,
}

// TODO: Impl Render
impl TitleEntry {

    pub fn gen_help_msg<'a>(&self) -> Paragraph<'a> {
        let (msg, style) = (
            vec![
                Span::raw("Press "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to submit title")
            ],
            Style::default(),
        );

        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);

        Paragraph::new(text)
    }

    pub fn gen_status_msg<'a>(&self) -> Paragraph<'a> {
        let validation_state_lock = self.title_validation_state.lock().unwrap();
        let span = match *validation_state_lock {
            TitleValidationState::Null => { Span::from(String::from("")) },
            TitleValidationState::Invalid => {
                Span::styled(
                    "\nInvalid Issue Title", 
                    Style::default().fg(RED)
                )
            },
            TitleValidationState::Validating => {
                Span::styled(
                    String::from("\nValidating"),
                    Style::default()
                )
            },
            TitleValidationState::Valid => {
                Span::styled(
                    "\nValidated", 
                    Style::default().fg(GREEN)
                )
            },
        };

        let text = Text::from(Spans::from(span));
        Paragraph::new(text)
    }

    pub fn render<B>(&self, f: &mut Frame<B>, area: Rect)
    where B: Backend,
    {
        // Split into two rows (top ==> help/status msg, bottom ==> input box)
        let row_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(area);

        let msg_col_layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(2)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(row_layout[0]);
        
        // render help msg
        f.render_widget(
            self.gen_help_msg()
                .block(Block::default().borders(Borders::ALL)),
            msg_col_layout[0]
        );

        f.render_widget(
            self.gen_status_msg()
                .block(Block::default().borders(Borders::ALL)),
            msg_col_layout[1]
        );

        // render input box
        self.input.render(f,row_layout[1]);
    }

}



impl Default for TitleEntry {
    fn default() -> TitleEntry {
        TitleEntry {
            input: UserInput::new(false),
            title_validation_state: Arc::new(Mutex::new(TitleValidationState::Null)),
        }
    }
}