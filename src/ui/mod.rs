

use std::fmt::Write;

use std::boxed::{
  Box
};

use crate::app;
use crate::Route;
use crate::util;

use app::App as App;

use crate::components::linear_team_select::LinearTeamSelectState;
use crate::components::linear_issue_display::LinearIssueDisplayState;
use crate::components::linear_workflow_state_display::LinearWorkflowStateDisplayState;


use tui::{
  backend::Backend,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans, Text},
  widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
  Frame,
};





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

    // info!("Calling get_rendered_teams_data with: {:?}", app.linear_team_select.teams_data);

    let mut items;
    let items_result;

    let handle = app.linear_team_select.teams_data.lock().unwrap();
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
    f.render_stateful_widget(items, chunks[0], &mut app.linear_team_select.teams_state);
}



pub fn draw_issue_display<B>(f: &mut Frame<B>, app: &mut App)
where 
  B: Backend,
{

  // info!("Calling draw_issue_display with: {:?}", app.linear_issue_display.issue_table_data);

  let issue_data_handle = app.linear_issue_display.issue_table_data.lock().unwrap();

  let table;
  let table_result = LinearIssueDisplayState::get_rendered_issue_data(&issue_data_handle);

  match table_result {
    Ok(x) => { table = x },
    Err(x) => {return;},
  }

  let mut table_state = app.linear_issue_display.issue_table_state.clone();

  // info!("table: {:?}", table);


  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(100)/*, Constraint::Percentage(35)*/].as_ref())
    .split(f.size());

  f.render_stateful_widget(table, chunks[0], &mut table_state);
  
  // Draw Workflow State Selection 
  if app.linear_draw_workflow_state_select == true {

    let workflow_states_handle = app.linear_workflow_select.workflow_states_data.lock().unwrap();

    // let block = Block::default().title("Popup").borders(Borders::ALL);
    let table;
    let table_result = LinearWorkflowStateDisplayState::get_rendered_workflow_state_select(&workflow_states_handle);

    match table_result {
      Ok(x) => { table = x },
      Err(x) => {return;},
    }

    let area = util::ui::centered_rect(40, 80, f.size());

    let workflow_chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                            .split(area);


    f.render_widget(Clear, area); //this clears out the background
    // info!("Cleared out space for Workflow States pop-up component");
    
    
    // let issue_data_handle = app.linear_issue_display.issue_table_data.lock().unwrap();
    let issue_data_inner;

    let issue_list_option = app.linear_issue_display.issue_table_state.selected();
    let issue_list_idx;

    // Check that an Issue is selected, if not:
    // don't render workflow pop-up and set linear_draw_workflow_state_select = false
    match issue_list_option {
      Some(x) => { issue_list_idx = x; },
      None => { 
        error!("Workflow States Component cannot render without an Issue selected");
        app.linear_draw_workflow_state_select = false;
        return; 
      }
    }

    // Check that issue list data exists, if not:
    // don't render workflow pop-up and set linear_draw_workflow_state_select = false
    match &*issue_data_handle {
      Some(x) => { issue_data_inner = x; },
      None => {
        error!("Workflow States Component cannot render without Issue Data successfully loaded");
        app.linear_draw_workflow_state_select = false;
        return;
      }
    }

    // Get 'color', 'number', and 'title' field from selected Issue
    let mut selected_issue = issue_data_inner.get(&issue_list_idx);

    let mut selected_issue_color = selected_issue.get_or_insert(&serde_json::Value::Null).get("state");
    // let mut selected_issue_color = selected_issue_color.get("color");
    let mut selected_issue_color = selected_issue_color.get_or_insert(&serde_json::Value::Null).get("color");

    let selected_issue_color = selected_issue_color.get_or_insert(&serde_json::Value::Null);





    let mut selected_issue_number = selected_issue.get_or_insert(&serde_json::Value::Null).get("number");
    // let mut selected_issue_number = selected_issue_number.get("number");
    let selected_issue_number = selected_issue_number.get_or_insert(&serde_json::Value::Null);

    let mut selected_issue_title = selected_issue.get_or_insert(&serde_json::Value::Null).get("title");
    // let mut selected_issue_title = selected_issue_title.get("title");
    let selected_issue_title = selected_issue_title.get_or_insert(&serde_json::Value::Null);


    /*
    let selected_issue_number = issue_data_inner.get(issue_list_idx)
                                                .get_or_insert(&serde_json::Value::Null)
                                                .get("number")
                                                .get_or_insert(&serde_json::Value::Null);

    let selected_issue_title = issue_data_inner.get(issue_list_idx)
                                                .get_or_insert(&serde_json::Value::Null)
                                                .get("title")
                                                .get_or_insert(&serde_json::Value::Null);
    */
    // Return if one of the issue's fields was not found
    if  selected_issue_color.is_null() ||
        selected_issue_number.is_null() ||
        selected_issue_title.is_null()
    {
          error!("Workflow States Component failed to acquire all required Issue fields - color: {:?}, number: {:?}, title: {:?}", selected_issue_color, selected_issue_number, selected_issue_title);
          app.linear_draw_workflow_state_select = false;
          return;
    }


    let mut final_title = String::new();
    
    write!(&mut final_title, "{} - {}", selected_issue_number.as_i64().get_or_insert(-0), selected_issue_title.as_str().get_or_insert("ERR TITLE NOT FOUND"));

    // info!("Workflow State Selection final Issue Title: {}", final_title);

    let mut title_color = util::ui::style_color_from_hex_str(selected_issue_color);

    // Render Issue title in upper chunk
    let text = vec![
      Spans::from(Span::styled(
        final_title,
        Style::default().fg(*title_color.get_or_insert(Color::Green)).add_modifier(Modifier::ITALIC),
      ))
    ];

    let paragraph = Paragraph::new(text.clone())
    .block(Block::default()/*.title("Left Block")*/.borders(Borders::ALL))
    .alignment(Alignment::Left).wrap(Wrap { trim: true });

    f.render_widget(paragraph, workflow_chunks[0]);


    let mut table_state = app.linear_workflow_select.workflow_states_state.clone();

    // Render workflow state selection table in lower chunk
    f.render_stateful_widget(table, workflow_chunks[1], &mut table_state);
    

  }
}