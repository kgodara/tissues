use tui::{
    backend::Backend,
    layout::{Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct UserInput {
    pub input: String,
    pub is_secret: bool,
    pub cursor_offset: usize,
}

pub fn new(is_secret: bool) -> UserInput{
    UserInput {
        input: String::new(),
        is_secret,
        cursor_offset: 0,
    }
}

impl UserInput {

    pub fn new(is_secret: bool) -> UserInput{
        UserInput {
            input: String::new(),
            is_secret,
            cursor_offset: 1,
        }
    }

    pub fn render<'a, B>(&self, f: &mut Frame<B>, rect: Rect)
    where B: Backend,
    {
        // Generate equivalent amount of '*' chars for each input char
        let grapheme_len: usize = self.input
            .graphemes(true)
            .count();

        let display_str: String = if self.is_secret { "*".repeat(grapheme_len) } else { self.input.to_string() };

        f.render_widget(
            Paragraph::new(display_str)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Input")),
            rect
        );
        
        f.set_cursor(
            // Put cursor past the end of the input text
            rect.x + self.cursor_offset as u16,
            // Move one line down, from the border to the input line
            rect.y + 1,
        )
    }

    pub fn unicode_width(&self) -> usize {
        unicode_width::UnicodeWidthStr::width(self.input.as_str())
    }

    pub fn set_input(&mut self, input_init: String) {
        self.input = input_init;
        // set initial cursor pos to end of input str
        self.cursor_offset = self.unicode_width() + 1;
    }

    pub fn insert(&mut self, ch: char) {
        let gr_inds = self.input.grapheme_indices(true)
              .collect::<Vec<(usize, &str)>>();

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
    }

    pub fn move_cursor_forwards(&mut self) {
        if self.cursor_offset <= self.unicode_width() { self.cursor_offset += 1; }
    }
}