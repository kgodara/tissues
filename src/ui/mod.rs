use std::sync::atomic::{ Ordering };

use crate::app;
use crate::util;

use app::{ App };

use crate::components::{
    dashboard_view_config_display::DashboardViewConfigDisplay,
    dashboard_view_panel::DashboardViewPanel,
    linear_custom_view_select::LinearCustomViewSelect,

    linear_issue_op_interface::{ LinearIssueOpInterface, ModificationOpData },
    linear_issue_modal,
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

use crate::linear::schema::CustomView;

use tui::{
  backend::Backend,
  layout::{Constraint, Direction, Layout, Rect, Alignment},
  style::{ Modifier, Style },
  text::{ Spans, Span },
  widgets::{Block, Borders, Clear, List, ListItem, TableState, Paragraph, Wrap},
  Frame,
};




pub const BASIC_VIEW_HEIGHT: u16 = 6;
pub const SMALL_TERMINAL_WIDTH: u16 = 150;
pub const SMALL_TERMINAL_HEIGHT: u16 = 45;

pub fn draw_config_interface<B>(f: &mut Frame<B>, app: & mut App)
where
  B: Backend,
{
    app.token_entry.render(f,app.loader_tick);

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
    // TODO: Re-enable this
    let viewer_obj_render_lock = app.viewer_obj_render.lock().unwrap();
    if let Some(viewer_obj) = &*viewer_obj_render_lock {
        let display_name = &viewer_obj.display_name;
        let org_name = &viewer_obj.organization.name;

        let mut viewer_label: String = String::new();
        viewer_label.push_str(display_name);
        viewer_label.push_str(" - ");
        viewer_label.push_str(org_name);

        let viewer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default());
    
        let viewer_p = Paragraph::new(Span::from(viewer_label))
            .block(viewer_block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        
        f.render_widget(viewer_p, chunks[0]);
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

    let mut layout_rects = ui::view_layout(num_views, chunks[2]);

    for (i, e) in view_panel_handle.iter().enumerate() {
        let view_data_handle = e.issue_table_data.lock().unwrap();

        // Get bounding-box for view panel
        let view_panel_rect = layout_rects.pop().unwrap();

        // subtract 2 from width to account for single character table borders
        let view_panel_content_rect = Rect::new(view_panel_rect.x, view_panel_rect.y, view_panel_rect.width-2, view_panel_rect.height);

        let widths: Vec<Constraint> = widths_from_rect( &view_panel_content_rect, &*VIEW_PANEL_COLUMNS);


        // Create TableStyle for ViewPanel
        let highlight_table: bool = 
            match app.linear_dashboard_view_panel_selected {
                Some(selected_idx) => selected_idx == i+1,
                None => false
            };

        // Get 'loading' bool from ViewPanel
        let loading_state: bool = e.loading.load(Ordering::Relaxed);


        // TODO: Create default color
        let view_panel_table_style = TableStyle { title_style: Some(( e.view.name.clone(), e.view.color.clone().unwrap_or("#000000".to_string()) )),
            row_bottom_margin: Some(0),
            view_idx: Some((i as u16)+1),
            highlight_table,
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
            let is_selected: bool = 
                match app.linear_dashboard_view_panel_selected {
                    Some(selected_view_panel_idx) => selected_view_panel_idx == (i+1),
                    None => false
                };

            // Determine the correct TableState, depending on if this view is selected or not
            let mut table_state = if is_selected { app.view_panel_issue_selected.clone().unwrap_or_default() } else { TableState::default() };

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

    // Draw Issue Expanded Modal
    if let Some(issue_obj) = &app.issue_to_expand {
        let area = util::ui::centered_rect(40, 40, f.size());

        let issue_modal_chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100),].as_ref())
            .split(area);
        
        f.render_widget(Clear, area); //this clears out the background

        linear_issue_modal::render_and_layout(f, issue_modal_chunk[0], issue_obj, app.scroll_tick);
    }


    // Draw Linear Issue Op Interface

    // IssueModificationOp::Title is not rendered with a table
    if app.modifying_issue && app.linear_issue_op_interface.current_op == Some(IssueModificationOp::Title) {
        let area = util::ui::centered_rect(50, 40, f.size());

        f.render_widget(Clear, area); //this clears out the background

        app.title_entry.render(f, area);
    }

    else if app.modifying_issue {

        let area = util::ui::centered_rect(40, 40, f.size());

        let current_op: &IssueModificationOp = if let Some(op) = &app.linear_issue_op_interface.current_op {
            op
        } else {
            error!("draw_action_select - app.linear_issue_op_interface.current_op must be Some(): {:?}", &app.linear_issue_op_interface.current_op);
            panic!("draw_action_select - app.linear_issue_op_interface.current_op must be Some(): {:?}", &app.linear_issue_op_interface.current_op)
        };

        let issue_op_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100),].as_ref())
            .split(area);

        f.render_widget(Clear, area); //this clears out the background

        let issue_op_widths: Vec<Constraint> = LinearIssueOpInterface::widths_from_rect_op(&issue_op_chunks[0], current_op);

        let issue_op_table_style = TableStyle {
            title_style: Some(( LinearIssueOpInterface::title_from_op(current_op),
                hex_str_from_style_color(&colors::ISSUE_MODIFICATION_TABLE_TITLE).unwrap_or_else(|| String::from("#000000"))
            )),
            row_bottom_margin: Some(0),
            view_idx: None,
            highlight_table: true,
            loading: app.linear_issue_op_interface.loading.load(Ordering::Relaxed),
            loader_state: app.loader_tick
        };

        let data_lock = app.linear_issue_op_interface.obj_data.lock().unwrap();

        let cloned_data: ModificationOpData = data_lock.clone();
        drop(data_lock);

        let mut issue_op_table = LinearIssueOpInterface::render(*current_op,
                &cloned_data,
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


pub fn draw_dashboard_view_config<B>(f: &mut Frame<B>, app: &mut App)
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
    let mut selected_view: Option<CustomView> = None;

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
        error!("draw_dashboard_view_config - app.dashboard_view_config_cmd_bar.render() failed");
        panic!("draw_dashboard_view_config - app.dashboard_view_config_cmd_bar.render() failed");
    }


    // Get Rects for DashboardViewConfigDisplay & CustomViewSelect
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



    // Draw Dashboard View Config Display

    // Create TableStyle for Dashboard View List
    let view_list_table_style = TableStyle { 
        title_style: 
        Some((
            String::from("Dashboard View Configuration"),
            hex_str_from_style_color(&colors::DASHBOARD_VIEW_LIST_TABLE_TITLE).unwrap_or_else(|| String::from("#000000")) ) ),
        row_bottom_margin: Some(0),
        view_idx: Some(1),
        highlight_table: app.linear_dashboard_view_list_selected,
        loading: false,
        loader_state: app.loader_tick,
    };

    // subtract 2 from width to account for single character table borders
    let view_display_content_rect = Rect::new(bottom_row_chunks[0].x, bottom_row_chunks[0].y, bottom_row_chunks[0].width-2, bottom_row_chunks[0].height);

    // let widths: Vec<Constraint> = widths_from_rect( &bottom_row_chunks[0], &*DASHBOARD_VIEW_CONFIG_COLUMNS);
    let widths: Vec<Constraint> = widths_from_rect( &view_display_content_rect, &*DASHBOARD_VIEW_CONFIG_COLUMNS);

    if let Ok(mut view_display_table) = 
        DashboardViewConfigDisplay::render(&app.linear_dashboard_view_list, &widths, view_list_table_style)
    {

        view_display_table = view_display_table.widths(&widths);


        let mut table_state = app.dashboard_view_display.view_table_state.clone();


        f.render_stateful_widget(view_display_table, bottom_row_chunks[0], &mut table_state);
    } else {
        error!("draw_dashboard_view_config - DashboardViewConfigDisplay::get_rendered_view_table failed");
        panic!("draw_dashboard_view_config - DashboardViewConfigDisplay::get_rendered_view_table failed");
    }


    // Draw Custom View Select
  
    let view_data_handle = &app.linear_custom_view_select.view_table_data.lock().unwrap();

    // Create TableStyle for Custom View Select
    let custom_view_select_table_style = TableStyle { 
        title_style: 
        Some((
            String::from("Custom View Select"),
            hex_str_from_style_color(&colors::CUSTOM_VIEW_SELECT_TABLE_TITLE).unwrap_or_else(|| String::from("#000000")) ) ),
        row_bottom_margin: Some(0),
        view_idx: Some(2),
        highlight_table: !app.linear_dashboard_view_list_selected,
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
        error!("draw_dashboard_view_config - LinearCustomViewSelect::get_rendered_view_data failed");
        panic!("draw_dashboard_view_config - LinearCustomViewSelect::get_rendered_view_data failed");
    }

}