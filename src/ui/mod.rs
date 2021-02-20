use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::App;
use crate::Route;
use crate::util;

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

  let team_fetch_result = app.linear_client.get_teams();
  let mut teams: serde_json::Value = serde_json::Value::Null;

  match team_fetch_result {
    Ok(x) => { teams = x; }
    Err(_) => return,
    _ => {}
  }

  // let Ok(teams) = team_fetch_result;

  println!("teams: {}", teams);


  match teams {
      serde_json::Value::Null => {
        // Reset back to previous screen, and continue past loop
        println!("Team Fetch failed");
        app.route = Route::ActionSelect;
        return; 
      },
      serde_json::Value::Array(_) => {},
      _ => {},
  }


  let teams_vec;

  match teams.as_array() {
    Some(x) => { teams_vec = x; },
    None => {
      return;
    }
  }

  let mut teams_list = util::StatefulList::with_items(teams_vec.clone());

  let items: Vec<ListItem> = teams_list
    .items
    .iter()
    .filter_map(|x| { x.as_str() })
    .map(|i| {
        let lines = vec![Spans::from(i)];
        ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        /*
        // let Some(i_val) = i;
        if let Some(i_val) = i {
          let lines = vec![Spans::from(i_val)];
          ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        } else {
          ListItem::new("Trash")
        }*/
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


    
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
      .split(f.size());

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut teams_list.state);

  

}