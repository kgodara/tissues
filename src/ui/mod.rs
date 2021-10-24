

use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::app;
use crate::util;

use app::{ App, InputMode };

use crate::components::{
    user_input::{ UserInput, InputContext, ValidationState, TitleValidationState, TokenValidationState },

    dashboard_view_display::DashboardViewDisplay,
    dashboard_view_panel::DashboardViewPanel,
    linear_custom_view_select::LinearCustomViewSelect,

    linear_issue_op_interface::LinearIssueOpInterface,
    linear_issue_modal::render_and_layout,

    task_status_modal,
    task_status_modal::TaskStatus,
};

use crate::util::{
    ui,
    ui::{ hex_str_from_style_color },
    table::{ TableStyle },
    dashboard::{fetch_selected_view_panel_issue, fetch_selected_view_panel_num},    
    layout::{ widths_from_rect },
};

use crate::constants::{
    colors,
    table_columns::{ DASHBOARD_VIEW_CONFIG_COLUMNS, CUSTOM_VIEW_SELECT_COLUMNS,
        VIEW_PANEL_COLUMNS },
    IssueModificationOp,
};

use tui::{
  backend::Backend,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders, Clear, List, ListItem, Paragraph, TableState, Wrap},
  Frame,
};

use serde_json::Value;




pub const BASIC_VIEW_HEIGHT: u16 = 6;
pub const SMALL_TERMINAL_WIDTH: u16 = 150;
pub const SMALL_TERMINAL_HEIGHT: u16 = 45;

pub fn draw_config_interface<B>(f: &mut Frame<B>, app: & mut App)
where
  B: Backend,
{

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Length(3),
                // Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());
    
    {
        let token_validation_state_lock = app.config_interface_input.token_validation_state.lock().unwrap();

        f.render_widget(UserInput::render_help_msg(&app.input_mode, InputContext::Token, &ValidationState::Token(token_validation_state_lock.clone())), chunks[0]);
        f.render_widget(UserInput::render_input_box(&app.config_interface_input.input, InputContext::Token, &app.input_mode), chunks[1]);
    }

    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + unicode_width::UnicodeWidthStr::width(app.config_interface_input.input.as_str()) as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let token_validation_state_lock = app.config_interface_input.token_validation_state.lock().unwrap();
    if *token_validation_state_lock == TokenValidationState::Validating {
        let task_area = util::ui::centered_rect(40, 20, f.size());

        f.render_widget(Clear, task_area); //this clears out the background

        let task_p: Paragraph = task_status_modal::render(TaskStatus::ValidatingToken, app.loader_tick);

        f.render_widget(task_p, task_area);
    }

}


pub fn draw_action_select<B>(f: &mut Frame<B>, app: & mut App)
where
  B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(5), Constraint::Percentage(20), Constraint::Percentage(70), Constraint::Percentage(5)].as_ref())
        .split(f.size());

    
    // Render the viewer displayName and organization name

    let linear_config_lock = app.linear_client.config.lock().unwrap();
    if let Some(viewer_obj) = &linear_config_lock.viewer_object {
        let display_name = &viewer_obj["displayName"];
        let org_name = &viewer_obj["organization"]["name"];

        if let Value::String(display_name_str) = display_name {
            if let Value::String(org_name_str) = org_name {

                let mut viewer_label: String = String::new();
                viewer_label.push_str(display_name_str);
                viewer_label.push_str(" - ");
                viewer_label.push_str(org_name_str);

                let viewer_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default());
            
                let viewer_p = Paragraph::new(Span::from(viewer_label))
                    .block(viewer_block)
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: true });
                
                f.render_widget(viewer_p, chunks[0]);
            }
        }
    }




    // Render the View Panel Command Bar

    // Determine which Commands are allowed based on state of selection
    let mut modify_title_cmd_active = false;
    let mut modify_workflow_state_cmd_active = false;
    let mut modify_assignee_cmd_active = false;
    let mut modify_project_cmd_active = false;
    let mut modify_cycle_cmd_active = false;
    let mut expand_issue_cmd_active = false;

    let mut refresh_cmd_active = false;


    // If a View Panel is selected && its not loading && issue is not expanded
    //     allow Refresh command
    if let Some(selected_view_panel_idx) = fetch_selected_view_panel_num(app) {
        // Fetch selected ViewPanel
        let view_panel_list_lock = app.linear_dashboard_view_panel_list.lock().unwrap();

        if let Some(x) = view_panel_list_lock.get(selected_view_panel_idx-1) {
            if !x.loading.load(Ordering::Relaxed) && app.issue_to_expand.is_none() {
                refresh_cmd_active = true;
            }
        }
        drop(view_panel_list_lock);
    }

    // If a View Panel Issue is selected && issue is not expanded, allow following commands
    if fetch_selected_view_panel_issue(app).is_some() && app.issue_to_expand.is_none() {
        modify_title_cmd_active = true;
        modify_workflow_state_cmd_active = true;
        modify_assignee_cmd_active = true;
        modify_project_cmd_active = true;
        modify_cycle_cmd_active = true;
        expand_issue_cmd_active = true;
    }

    // Update Command statuses
    app.view_panel_cmd_bar.set_modify_title_active(modify_title_cmd_active);
    app.view_panel_cmd_bar.set_modify_workflow_state_active(modify_workflow_state_cmd_active);
    app.view_panel_cmd_bar.set_modify_assignee_active(modify_assignee_cmd_active);
    app.view_panel_cmd_bar.set_modify_project_active(modify_project_cmd_active);
    app.view_panel_cmd_bar.set_modify_cycle_active(modify_cycle_cmd_active);
    app.view_panel_cmd_bar.set_expand_issue_active(expand_issue_cmd_active);


    app.view_panel_cmd_bar.set_refresh_panel_active(refresh_cmd_active);

    // Render command bar
    if let Ok(cmd_items) = app.view_panel_cmd_bar.render() {
        f.render_widget(cmd_items, chunks[1]);
    } else {
        error!("draw_action_select - app.view_panel_cmd_bar.render() failed");
        panic!("draw_action_select - app.view_panel_cmd_bar.render() failed");
    }

    
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

        // Get bounding-box for view panel
        let view_panel_rect = match num_views {
            1 => { ui::single_view_layout(i, chunks[2]) },
            2 => { ui::double_view_layout(i, chunks[2]) },
            3 => { ui::three_view_layout(i, chunks[2]) },
            4 => { ui::four_view_layout(i, chunks[2]) },
            5 => { ui::five_view_layout(i, chunks[2]) },
            6 => { ui::six_view_layout(i, chunks[2]) },
            _ => {continue;},
        };

        // subtract 2 from width to account for single character table borders
        let view_panel_content_rect = Rect::new(view_panel_rect.x, view_panel_rect.y, view_panel_rect.width-2, view_panel_rect.height);

        let widths: Vec<Constraint> = widths_from_rect( &view_panel_content_rect, &*VIEW_PANEL_COLUMNS);


        // Create TableStyle for ViewPanel

        let highlight_table: bool = 
            if let Some(selected_idx) = selected_view_idx {
                selected_idx == ((i as u16)+1)
            } else {
                false
            };

        // Get 'loading' bool from ViewPanel
        let loading_state: bool = e.loading.load(Ordering::Relaxed);


        let view_panel_table_style = TableStyle { title_style: Some(( e.filter["name"].clone(), e.filter["color"].clone() )),
            row_bottom_margin: Some(0),
            view_idx: Some((i as u16)+1),
            highlight_table,
            req_num: Some(req_num as u16),
            loading: loading_state,
            loader_state: app.loader_tick
        };


        if let Ok(mut view_panel_table) =
            DashboardViewPanel::render(&view_data_handle,
                &widths,
                view_panel_table_style
            )
        {

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

            view_panel_table = view_panel_table.widths(&widths);

            f.render_stateful_widget(view_panel_table, view_panel_rect, &mut table_state);
        }
    }

    drop(view_panel_handle);


    // Render the action list
    let items: Vec<ListItem> = app
      .actions
      .items
      .iter()
      .map(|i| {
          ListItem::new(vec![Spans::from(*i)])
      })
      .collect();


    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Action Select"))
        .highlight_style(
            // Style::default().add_modifier(Modifier::REVERSED);
            Style::default().add_modifier(Modifier::REVERSED),
        );

    f.render_stateful_widget(items, chunks[3], &mut app.actions.state);

    // If timezone load is occurring, draw task modal
    if app.team_tz_load_in_progress.load(Ordering::Relaxed) {
        let task_area = util::ui::centered_rect(40, 20, f.size());

        f.render_widget(Clear, task_area); //this clears out the background

        let task_p: Paragraph = task_status_modal::render(TaskStatus::LoadingTeamTimezones, app.loader_tick);

        f.render_widget(task_p, task_area);
    }

    // Draw Issue Expanded Modal
    if let Some(issue_obj) = &app.issue_to_expand {
        let area = util::ui::centered_rect(40, 40, f.size());

        let issue_modal_chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100),].as_ref())
            .split(area);
        
        f.render_widget(Clear, area); //this clears out the background

        render_and_layout(f, issue_modal_chunk[0], issue_obj, app.scroll_tick);
    }


    // Draw Linear Issue Op Interface

    // IssueModificationOp::Title is not rendered with a table
    if app.modifying_issue && app.linear_issue_op_interface.current_op == IssueModificationOp::Title {
        let area = util::ui::centered_rect(50, 40, f.size());

        let input_chunks = Layout::default()
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

        f.render_widget(Clear, area); //this clears out the background

        f.render_widget(
            UserInput::render_help_msg(&app.input_mode, InputContext::IssueTitle, &ValidationState::Title(app.issue_title_input.title_validation_state))
                .block(Block::default().borders(Borders::ALL)),
            input_chunks[0]);
        f.render_widget(UserInput::render_input_box(&app.issue_title_input.input, InputContext::IssueTitle, &app.input_mode), input_chunks[1]);

        match app.input_mode {
            InputMode::Normal =>
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                {}
    
            InputMode::Editing => {
                // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                f.set_cursor(
                    // Put cursor past the end of the input text
                    input_chunks[1].x + unicode_width::UnicodeWidthStr::width(app.issue_title_input.input.as_str()) as u16 + 1,
                    // Move one line down, from the border to the input line
                    input_chunks[1].y + 1,
                )
            }
        }
    }

    else if app.modifying_issue {

        let area = util::ui::centered_rect(40, 40, f.size());

        let issue_op_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100),].as_ref())
            .split(area);

        f.render_widget(Clear, area); //this clears out the background

        let issue_op_widths: Vec<Constraint> = LinearIssueOpInterface::widths_from_rect_op(&issue_op_chunks[0], &app.linear_issue_op_interface.current_op);

        let issue_op_table_style = TableStyle {
            title_style: Some(( Value::String(LinearIssueOpInterface::title_from_op(&app.linear_issue_op_interface.current_op)),
                Value::String(hex_str_from_style_color(&colors::ISSUE_MODIFICATION_TABLE_TITLE).unwrap_or_else(|| String::from("#000000")))
            )),
            row_bottom_margin: Some(0),
            view_idx: None,
            highlight_table: true,
            req_num: None,
            loading: app.linear_issue_op_interface.loading.load(Ordering::Relaxed),
            loader_state: app.loader_tick
        };

        let data_handle = app.linear_issue_op_interface.table_data_from_op();
        let data_lock = data_handle.lock().unwrap();

        let cloned_data_vec: Vec<Value> = data_lock.clone();
        drop(data_lock);

        let mut issue_op_table = LinearIssueOpInterface::render(app.linear_issue_op_interface.current_op,
                &cloned_data_vec,
                &issue_op_widths,
                issue_op_table_style
            )
            .to_owned()
            .unwrap();

        let mut table_state = app.linear_issue_op_interface.data_state.clone();

        issue_op_table = issue_op_table.widths(&issue_op_widths);

        // Render IssueOp table in lower chunk
        f.render_stateful_widget(issue_op_table, issue_op_chunks[0], &mut table_state);
    }
}


pub fn draw_dashboard_view_display<B>(f: &mut Frame<B>, app: &mut App)
where
  B: Backend,
{


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
    let mut remove_view_cmd_active = false;

    // If a View is not selected, no Commands allowed
    if view_is_selected {
        // A filled view slot is selected, allow Replace View and Remove View Commands
        if selected_view.is_some() {
            remove_view_cmd_active = true;
        }
    }

    // Update Command statuses
    app.dashboard_view_config_cmd_bar.set_remove_view_active(remove_view_cmd_active);

    // Render command bar
    if let Ok(cmd_items) = app.dashboard_view_config_cmd_bar.render() {
        f.render_widget(cmd_items, chunks[0]);
    } else {
        error!("draw_dashboard_view_display - app.dashboard_view_config_cmd_bar.render() failed");
        panic!("draw_dashboard_view_display - app.dashboard_view_config_cmd_bar.render() failed");
    }


    // Get Rects for DashboardViewDisplay & CustomViewSelect
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



    // Draw Dashboard View Display

    // Create TableStyle for Dashboard View List
    let view_list_table_style = TableStyle { 
        title_style: 
        Some((
            Value::String(String::from("Dashboard View Configuration")),
            Value::String( hex_str_from_style_color(&colors::DASHBOARD_VIEW_LIST_TABLE_TITLE).unwrap_or_else(|| String::from("#000000")) ) )),
        row_bottom_margin: Some(0),
        view_idx: Some(1),
        highlight_table: app.linear_dashboard_view_list_selected,
        req_num: None,
        loading: false,
        loader_state: app.loader_tick,
    };

    // subtract 2 from width to account for single character table borders
    let view_display_content_rect = Rect::new(bottom_row_chunks[0].x, bottom_row_chunks[0].y, bottom_row_chunks[0].width-2, bottom_row_chunks[0].height);

    // let widths: Vec<Constraint> = widths_from_rect( &bottom_row_chunks[0], &*DASHBOARD_VIEW_CONFIG_COLUMNS);
    let widths: Vec<Constraint> = widths_from_rect( &view_display_content_rect, &*DASHBOARD_VIEW_CONFIG_COLUMNS);

    if let Ok(mut view_display_table) = 
        DashboardViewDisplay::render(&app.linear_dashboard_view_list, &widths, view_list_table_style)
    {

        view_display_table = view_display_table.widths(&widths);


        let mut table_state = app.dashboard_view_display.view_table_state.clone();


        f.render_stateful_widget(view_display_table, bottom_row_chunks[0], &mut table_state);
    } else {
        error!("draw_dashboard_view_display - DashboardViewDisplay::get_rendered_view_table failed");
        panic!("draw_dashboard_view_display - DashboardViewDisplay::get_rendered_view_table failed");
    }


    // Draw Custom View Select
  
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
        req_num: None,
        loading: app.linear_custom_view_select.loading.load(Ordering::Relaxed),
        loader_state: app.loader_tick,
    };

    // subtract 2 from width to account for single character table borders
    let view_select_content_rect = Rect::new(bottom_row_chunks[1].x, bottom_row_chunks[1].y, bottom_row_chunks[1].width-2, bottom_row_chunks[1].height);

    // lazy_static! provides a struct which dereferences towards target struct, hence: '&*'
    // https://github.com/rust-lang-nursery/lazy-static.rs/issues/119#issuecomment-419595818
    let widths: Vec<Constraint> = widths_from_rect( &view_select_content_rect, &*CUSTOM_VIEW_SELECT_COLUMNS);

    if let Ok(mut view_select_table) = LinearCustomViewSelect::render(view_data_handle,
        &widths,
        custom_view_select_table_style)
    {
        view_select_table = view_select_table.widths(&widths);

        let mut custom_view_table_state = app.linear_custom_view_select.view_table_state.clone();

        f.render_stateful_widget(view_select_table, bottom_row_chunks[1], &mut custom_view_table_state);
    } else {
        error!("draw_dashboard_view_display - LinearCustomViewSelect::get_rendered_view_data failed");
        panic!("draw_dashboard_view_display - LinearCustomViewSelect::get_rendered_view_data failed");
    }

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

}