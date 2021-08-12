

use std::fmt::Write;

use std::boxed::{
  Box
};

use crate::app;
use crate::Route;
use crate::util;

use app::App as App;

use crate::components::dashboard_view_display::DashboardViewDisplay;
use crate::components::dashboard_view_panel::DashboardViewPanel;

use crate::components::linear_custom_view_select::LinearCustomViewSelect;
use crate::components::linear_team_select::LinearTeamSelectState;
use crate::components::linear_issue_display::LinearIssueDisplay;
use crate::components::linear_workflow_state_display::LinearWorkflowStateDisplayState;

use crate::util::colors;
use crate::util::ui;

use crate::util::ui::{ TableStyle, style_color_from_hex_str, hex_str_from_style_color };
use crate::util::{ state_list, state_table };

use crate::util::fetch_selected_view_panel_issue;


use tui::{
  backend::Backend,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans, Text},
  widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Row, Table, TableState, Wrap},
  Frame,
};

use serde_json::Value;





pub const BASIC_VIEW_HEIGHT: u16 = 6;
pub const SMALL_TERMINAL_WIDTH: u16 = 150;
pub const SMALL_TERMINAL_HEIGHT: u16 = 45;



pub fn draw_action_select<B>(f: &mut Frame<B>, app: & mut App)
where
  B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(70), Constraint::Percentage(10)].as_ref())
        .split(f.size());
    
    // Render the View Panel Command Bar
    

    
    // Iterate through the list of View Panels & render each to the appropriate position within layour

    let view_panel_handle = app.linear_dashboard_view_panel_list.lock().unwrap();
    let num_views = view_panel_handle.len();

    for (i, e) in view_panel_handle.iter().enumerate() {
        let view_data_handle = e.issue_table_data.lock().unwrap();
        let selected_view_idx = if let Some(selected_idx) = app.linear_dashboard_view_panel_selected { Some(selected_idx as u16)}
                                    else {None};

        // Clone request_num to a u32
        let req_num_handle = e.request_num.lock().unwrap();
        let req_num: u32 = *req_num_handle;
        drop(req_num_handle);
        
        let view_table_result = DashboardViewPanel::render(&view_data_handle,
                                                            &e.filter,
                                                            req_num,
                                                            i as u16,
                                                            &selected_view_idx
                                                            );

        // Determine if this view panel is currently selected
        let mut is_selected = false;
        if let Some(selected_view_panel_idx) = app.linear_dashboard_view_panel_selected {
            if selected_view_panel_idx == (i+1) {
                is_selected = true;
            }
        }

        // Determine the correct TableState, depending on if this view is selected or not
        let table_state_option = if is_selected { app.view_panel_issue_selected.clone() } else { None };

        let mut table_state = match table_state_option {
            Some(table_state_val) => { table_state_val },
            None => { TableState::default() }
        };

        if let Ok(view_table) = view_table_result {
            match num_views {
                0 => {},
                1 => { f.render_stateful_widget(view_table, ui::single_view_layout(i, chunks[1]), &mut table_state); },
                2 => { f.render_stateful_widget(view_table, ui::double_view_layout(i, chunks[1]), &mut table_state); },
                3 => { f.render_stateful_widget(view_table, ui::three_view_layout(i, chunks[1]), &mut table_state); }
                4 => { f.render_stateful_widget(view_table, ui::four_view_layout(i, chunks[1]), &mut table_state); },
                5 => { f.render_stateful_widget(view_table, ui::five_view_layout(i, chunks[1]), &mut table_state) },
                _ => { f.render_stateful_widget(view_table, ui::six_view_layout(i, chunks[1]), &mut table_state)},
            }
        }
    }

    drop(view_panel_handle);


    // Render the action list
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
        .block(Block::default().borders(Borders::ALL).title("Action Select"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, chunks[2], &mut app.actions.state);

    // Draw Workflow State Selection 
    if app.linear_draw_workflow_state_select {
        // debug!("draw_action_select - app.linear_draw_workflow_state_select is true");

        let table;
        let table_result;
        let workflow_states_handle = app.linear_workflow_select.workflow_states_data.lock().unwrap();

        let cloned_states_vec: Vec<Value> = workflow_states_handle.clone();
        table_result = LinearWorkflowStateDisplayState::get_rendered_workflow_state_select(&cloned_states_vec).to_owned();
        drop(workflow_states_handle);

        match table_result {
            Ok(x) => { table = x },
            Err(_) => {return;},
        }

        // debug!("draw_action_select - get_rendered_workflow_state_select() success");

        let area = util::ui::centered_rect(40, 80, f.size());

        let workflow_chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                                .split(area);


        f.render_widget(Clear, area); //this clears out the background

        // debug!("draw_action_select - about to call fetch_selected_view_panel_issue()");

        let selected_issue_opt = fetch_selected_view_panel_issue(&app);
        let selected_issue;

        // debug!("draw_action_select - fetch_selected_view_panel_issue() success");


        // Check that an Issue is selected, if not:
        // don't render workflow pop-up and set linear_draw_workflow_state_select = false
        if let Some(x) = selected_issue_opt {
            selected_issue = x;
        }
        else {
            error!("Workflow States Component cannot render without an Issue selected");
            app.linear_draw_workflow_state_select = false;
            return;
        }
        // Get 'color', 'number', and 'title' field from selected Issue

        let selected_issue_color: Value = selected_issue["state"]["color"].clone();
        let selected_issue_number: Value = selected_issue["number"].clone();
        let selected_issue_title: Value = selected_issue["title"].clone();

        // Return if one of the issue's fields was not found
        if  selected_issue_color.is_null() ||
            selected_issue_number.is_null() ||
            selected_issue_title.is_null()
        {
            error!("Workflow States Component failed to acquire all required Issue fields - color: {:?}, number: {:?}, title: {:?}", selected_issue_color, selected_issue_number, selected_issue_title);
            app.linear_draw_workflow_state_select = false;
            return;
        }

        // debug!("draw_action_select - Get 'color', 'number', and 'title' field from selected Issue success");



        let mut final_title = String::new();
        
        write!( &mut final_title,
                "{} - {}",
                selected_issue_number.as_i64().get_or_insert(-0),
                selected_issue_title.as_str().get_or_insert("ERR TITLE NOT FOUND")
        );

        // info!("Workflow State Selection final Issue Title: {}", final_title);

        let mut title_color = util::ui::style_color_from_hex_str(&selected_issue_color);

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
        
        // debug!("draw_action_select - rendering workflow state components");
        
        f.render_widget(paragraph, workflow_chunks[0]);

        let mut table_state = app.linear_workflow_select.workflow_states_state.clone();

        // Render workflow state selection table in lower chunk
        f.render_stateful_widget(table, workflow_chunks[1], &mut table_state);
    }
}

pub fn draw_dashboard_view_display<B>(f: &mut Frame<B>, app: &mut App)
where
  B: Backend,
{

  // Create TableStyle for Dashboard View List
  let view_list_table_style = TableStyle { 
    title_style: 
      Some((
        Value::String(String::from("Dashboard View Configuration")),
        Value::String( hex_str_from_style_color(&colors::DASHBOARD_VIEW_LIST_TABLE_TITLE).unwrap_or_else(|| String::from("#000000")) ) )),
    row_bottom_margin: Some(0),
    view_idx: Some(1),
    highlight_table: app.linear_dashboard_view_list_selected,
    req_num: None
  };

  let table;
  let table_result = DashboardViewDisplay::get_rendered_view_table(&app.linear_dashboard_view_list, view_list_table_style);

  match table_result {
    Ok(x) => { table = x },
    Err(x) => {return;},
  }

  let mut table_state = app.dashboard_view_display.view_table_state.clone();

  // info!("table: {:?}", table);


  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
    .split(f.size());

  // Draw Command Bar to display Applicable Commands

  // Get Selected Custom View from app.linear_dashboard_view_list using app.linear_dashboard_view_idx
  let mut view_is_selected = false;
  let mut selected_view: Option<Value> = None;

  if let Some(view_idx) = app.linear_dashboard_view_idx {
    view_is_selected = true;
    selected_view = app.linear_dashboard_view_list[view_idx].clone();
  }

  // Determine which Commands are allowed based on state of selection
  // let mut add_view_cmd_active = false;
  // let mut replace_view_cmd_active = false;
  let mut remove_view_cmd_active = false;

  // If a View is not selected, no Commands allowed
  if view_is_selected {
    // A filled view slot is selected, allow Replace View and Remove View Commands
    if let Some(view) = selected_view {
      // replace_view_cmd_active = true;
      remove_view_cmd_active = true;
    }
    /*
    // An empty view slot is selected, only Add View Command is allowed
    else {
      // add_view_cmd_active = true;
    }
    */
  }

  debug!("remove_view_cmd_active: {:?}", remove_view_cmd_active);
  app.dashboard_view_config_cmd_bar.set_remove_view_active(remove_view_cmd_active);

  let cmd_items;
  let cmd_items_result = app.dashboard_view_config_cmd_bar.render();

  match cmd_items_result {
    Ok(x) => { cmd_items = x },
    Err(x) => {return;},
  }

  f.render_widget(cmd_items, chunks[0]);

  let bottom_row_chunks = Layout::default()
                          .direction(Direction::Horizontal)
                          .constraints(
                              [
                                  Constraint::Percentage(50),
                                  Constraint::Percentage(50),
                              ]
                              .as_ref(),
                          )
                          .split(chunks[1]);

  f.render_stateful_widget(table, bottom_row_chunks[0], &mut table_state);

  let view_table;
  
  let view_data_handle = &app.linear_custom_view_select.view_table_data.lock().unwrap();

  // Create TableStyle for Custom View Select
  let custom_view_select_table_style = TableStyle { 
    title_style: 
      Some((
        Value::String(String::from("Custom View Select")), 
        Value::String( hex_str_from_style_color(&colors::CUSTOM_VIEW_SELECT_TABLE_TITLE).unwrap_or_else(|| String::from("#000000")) ) )),
    row_bottom_margin: Some(0),
    view_idx: Some(2),
    highlight_table: !app.linear_dashboard_view_list_selected,
    req_num: None
  };

  let view_table_result = LinearCustomViewSelect::get_rendered_view_data(view_data_handle, custom_view_select_table_style);
  match view_table_result {
    Ok(x) => { view_table = x },
    Err(x) => {return;},
  }

  let mut custom_view_table_state;

  /*
  if None == app.linear_custom_view_select.view_table_state.selected() {
    let custom_view_select_handle = app.linear_custom_view_select.view_table_data.lock().unwrap();

    if let Some(custom_view_data) = &*custom_view_select_handle {
      if let Value::Array(custom_view_vec) = custom_view_data {
          if custom_view_vec.len() > 0 {
              let mut table_state = TableState::default();
              state_table::next(&mut table_state, &custom_view_vec);
              app.linear_custom_view_select.view_table_state = table_state.clone();
          }
      }
    }
  }
  */
  custom_view_table_state = app.linear_custom_view_select.view_table_state.clone();

  
  f.render_stateful_widget(view_table, bottom_row_chunks[1], &mut custom_view_table_state);
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


/*
pub fn draw_issue_display<B>(f: &mut Frame<B>, app: &mut App)
where 
  B: Backend,
{

  // info!("Calling draw_issue_display with: {:?}", app.linear_issue_display.issue_table_data);

  let issue_data_handle = app.linear_issue_display.issue_table_data.lock().unwrap();

  let table;

  let table_style = TableStyle { title_style: None, row_bottom_margin: Some(0), view_idx: None, selected_view_idx: None, req_num: None };

  let table_result = LinearIssueDisplay::get_rendered_issue_data(&issue_data_handle, table_style);

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
*/