// TODO: Assert that all tokens entered into user_input component are visible ASCII characters (32-127)

use tui::{
    backend::Backend,
    layout::{ Constraint, Direction, Layout },
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{ Paragraph },
    Frame,
};

use std::sync::{ Arc, Mutex };

use crate::components::user_input::UserInput;

use crate::constants::colors::{ RED, GREEN };

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenValidationState {
    Null,
    Invalid,
    Validating,
    Valid,
}



#[derive(Debug)]
pub struct TokenEntry {
    pub input: UserInput,
    pub token_validation_state: Arc<Mutex<TokenValidationState>>,
}

// TODO: Impl Render
impl TokenEntry {

    pub fn gen_help_msg<'a>(&self) -> Paragraph<'a> {
        let (msg, style) = (
            vec![
                Span::raw("Press "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to submit token")
            ],
            Style::default(),
        );

        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);

        Paragraph::new(text)
    }


    pub fn gen_status_msg(&self, loader_tick: u16) -> Option<Span<'static>> {
        let validation_state_lock = self.token_validation_state.lock().unwrap();
        match *validation_state_lock {
            TokenValidationState::Null => { None },
            TokenValidationState::Invalid => {
                Some(Span::styled(
                    "\nInvalid Linear Access Token", 
                    Style::default().fg(RED)
                ))
            },
            TokenValidationState::Validating => {
                Some(Span::styled(
                    String::from("\nValidating") + match loader_tick%3 {
                        0 => ".",
                        1 => "..",
                        2 => "...",
                        _ => unreachable!()
                    },
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
    }

    pub fn render<B>(&self, f: &mut Frame<B>, loader_tick: u16, )
    where B: Backend,
    {
        // Split into two rows (top ==> help/status msg, bottom ==> input box)
        let row_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Max(1),
                    Constraint::Max(1),
                ]
                .as_ref(),
            )
            .split(f.size());

        let msg_col_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(row_layout[0]);

        // render help msg
        f.render_widget(self.gen_help_msg(), msg_col_layout[0]);

        // render status msg
        if let Some(msg) = self.gen_status_msg(loader_tick) {
            f.render_widget(Paragraph::new(msg), msg_col_layout[1]);
        }

        // render input box
        self.input.render(f,row_layout[1]);
    }

}


impl Default for TokenEntry {
    fn default() -> TokenEntry {
        TokenEntry {
            input: UserInput::new(true),
            token_validation_state: Arc::new(Mutex::new(TokenValidationState::Null)),
        }
    }
}