use tui::{
    layout::{ Alignment },
    style::{ Style },
    text::{ Span },
    widgets::{ Paragraph, Wrap, Block, Borders },
};

pub enum TaskStatus {
    LoadingTeamTimezones,
}

pub fn render<'a>(task: TaskStatus, loader_tick: u16 ) -> Paragraph<'a> {

    let mut task_text: String = match task {
        TaskStatus::LoadingTeamTimezones => String::from("Loading Team Timezones")
    };

    match loader_tick%3 {
        0 => task_text.push('.'),
        1 => task_text.push_str(".."),
        2 => task_text.push_str("..."),
        _ => unreachable!()
    };

    let task_block = Block::default()
    .borders(Borders::ALL)
    .border_style(Style::default());

    let task_p = Paragraph::new(Span::from(task_text))
        .block(task_block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    task_p
}
