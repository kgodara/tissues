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
use crate::components::linear_issue_display::LinearIssueDisplayState;
use crate::components::linear_workflow_state_display::LinearWorkflowStateDisplayState;



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

    // info!("Calling get_rendered_teams_data with: {:?}", app.linear_team_select_state.teams_data);

    let mut items;
    let items_result;

    let handle = app.linear_team_select_state.teams_data.lock().unwrap();
    items_result = LinearTeamSelectState::get_rendered_teams_data(&*handle);


    match items_result {
      Ok(x) => { items = x },
      Err(_) => {return;},
    }


    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
      .split(f.size());

    // info!("About to render team select");
    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.linear_team_select_state.teams_state);
}



pub fn draw_issue_display<B>(f: &mut Frame<B>, app: &mut App)
where 
  B: Backend,
{

  // info!("Calling draw_issue_display with: {:?}", app.linear_issue_display_state.issue_table_data);

  let handle = app.linear_issue_display_state.issue_table_data.lock().unwrap();

  let table;
  let table_result = LinearIssueDisplayState::get_rendered_issue_data(&handle);

  match table_result {
    Ok(x) => { table = x },
    Err(x) => {return;},
  }

  let mut table_state = app.linear_issue_display_state.issue_table_state.clone();

  // info!("table: {:?}", table);


  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(100)/*, Constraint::Percentage(35)*/].as_ref())
    .split(f.size());

  f.render_stateful_widget(table, chunks[0], &mut table_state);

  if app.linear_draw_workflow_state_select == true {

    let handle = app.linear_workflow_select_state.workflow_states_data.lock().unwrap();

    // let block = Block::default().title("Popup").borders(Borders::ALL);
    let table;
    let table_result = LinearWorkflowStateDisplayState::get_rendered_workflow_state_select(&handle);

    match table_result {
      Ok(x) => { table = x },
      Err(x) => {return;},
    }

    let area = util::ui::centered_rect(60, 60, f.size());
    f.render_widget(Clear, area); //this clears out the background
    // f.render_widget(table, area);

    let mut table_state = app.linear_workflow_select_state.workflow_states_state.clone();

    f.render_stateful_widget(table, area, &mut table_state);

  }
}