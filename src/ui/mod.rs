use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
    Frame,
};

use std::boxed::{
  Box
};

use crate::App;
use crate::Route;
use crate::util;
use crate::components::linear_team_select::LinearTeamSelectState;

pub const BASIC_VIEW_HEIGHT: u16 = 6;
pub const SMALL_TERMINAL_WIDTH: u16 = 150;
pub const SMALL_TERMINAL_HEIGHT: u16 = 45;



pub fn draw_action_select<B>(f: &mut Frame<B>, app: &mut App)
where
  B: Backend,
{
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
    .split(f.size());

    let items: Vec<ListItem> = app
      .actions
      .items
      .iter()
      .map(|i| {
          let mut lines = vec![Spans::from(*i)];
          /*
          for _ in 0..i.1 {
              lines.push(Spans::from(Span::styled(
                  "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
                  Style::default().add_modifier(Modifier::ITALIC),
              )));
          }
          */
          ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
      })
      .collect();

      // Create a List from all list items and highlight the currently selected one
      let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

      // We can now render the item list
      f.render_stateful_widget(items, chunks[0], &mut app.actions.state);
}



pub fn draw_team_select<B>(f: &mut Frame<B>, app: &mut App)
where
  B: Backend,
{

    info!("Calling get_rendered_teams_data_2 with: {:?}", app.linear_team_select_state.teams_data);

    let items;
    let items_result = LinearTeamSelectState::get_rendered_teams_data(&app.linear_team_select_state.teams_data);

    match items_result {
      Ok(x) => { items = x },
      Err(x) => {return;},
    }

    
    let list_state_result = app.linear_team_select_state.teams_stateful.as_mut();
    let list_state;


    match list_state_result {
      Ok(x) => { list_state = x },
      Err(x) => {return;},
    }

    // info!("items: {:?}", items);


    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
      .split(f.size());

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut list_state.state);
    
}