
use termion::{event::Key,};

use crate::app::{App, Platform, Route};
use crate::network::IOEvent;
use crate::util::{ state_list, state_table };

use crate::components::linear_workflow_state_display::LinearWorkflowStateDisplayState;

use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::Sender;

use serde_json::Value;

use tui::{
    widgets::{ TableState },
};

#[derive(Debug)]
pub enum Command {

    // Arow Key Commands
    MoveBack,
    ScrollDown,
    ScrollUp,
    Confirm,

    // Char Commands
    Quit,
    Delete,
    SelectViewPanel(usize),
    SelectDashboardViewList,
    SelectCustomViewSelect,


    OpenLinearWorkflowStateSelection,

}


pub fn get_cmd(cmd_str: &mut String, input: Key, current_route: &Route) -> Option<Command> {
    match input {
        // Navigation/Confirmation related inputs
        // These will always clear the command string
        Key::Left => {
            Some(Command::MoveBack)
        },
        Key::Down => {
            Some(Command::ScrollDown)
        },
        Key::Up => {
            Some(Command::ScrollUp)
        },
        Key::Right => {
            Some(Command::Confirm)
        },

        // Contextual User commands
        Key::Char(ch) => {
            cmd_str.push(ch);
            match cmd_str.as_str() {
                // Quit Command
                "q" => {
                    Some(Command::Quit)
                },
                // Delete Command
                "d" => {
                    Some(Command::Delete)
                },
                // Modify Command
                "m" => {
                    Some(Command::OpenLinearWorkflowStateSelection)
                },

                // View Panel Selection Shortcuts
                "1" => {
                    match current_route {
                        Route::DashboardViewDisplay => {
                            Some(Command::SelectDashboardViewList)
                        },
                        _ => {
                            Some(Command::SelectViewPanel(1))
                        }
                    }
                },
                "2" => {
                    match current_route {
                        Route::DashboardViewDisplay => {
                            Some(Command::SelectCustomViewSelect)
                        },
                        _ => {
                            Some(Command::SelectViewPanel(2))
                        }
                    }
                },
                "3" => {
                    Some(Command::SelectViewPanel(3))
                },
                "4" => {
                    Some(Command::SelectViewPanel(4))
                },
                "5" => {
                    Some(Command::SelectViewPanel(5))
                },
                "6" => {
                    Some(Command::SelectViewPanel(6))
                },

                _ => {
                    None
                }
            }
        },

        _ => { None }
    }
}

pub async fn exec_add_cmd(app: &mut App<'_>) {

    info!("Executing 'add' command");

    if Route::DashboardViewDisplay == app.route {
        // Verify that an empty slot is selected
        // if so, switch to the CustomViewSelect Route to allow for selection of a Custom View to add
        let selected_view: Option<Value>;
        
        if let Some(view_idx) = app.linear_dashboard_view_idx {

            selected_view = app.linear_dashboard_view_list[view_idx].clone();

            // An empty view slot is selected
            if selected_view.is_none() {
                // app.change_route(Route::CustomViewSelect, &tx);
                exec_select_custom_view_select_cmd(app);
            }

        }
    }
}

pub async fn exec_replace_cmd(app: &mut App<'_>) {
    info!("Executing 'replace' command");
    
    // User is attempting to replace a Custom View with a new Custom View on the Dashboard
    if Route::DashboardViewDisplay == app.route {
        // Verify that a populated slot is selected
        // if so, switch to the CustomViewSelect Route to allow for selection of a Custom View to add
        let selected_view: Option<Value>;
        if let Some(view_idx) = app.linear_dashboard_view_idx {
            selected_view = app.linear_dashboard_view_list[view_idx].clone();

            // A populated view slot is selected
            if selected_view.is_some() {
                // app.change_route(Route::CustomViewSelect, &tx);
                exec_select_custom_view_select_cmd(app);
            }
        }
    }
}

pub async fn exec_delete_cmd(app: &mut App<'_>) {
    info!("Executing 'delete' command");

    // User is attempting to remove a Custom View on the Dashboard
    if Route::DashboardViewDisplay == app.route {
        // Verify that a populated slot is selected
        // if so, set it to None
        let selected_view: Option<Value>;
        if let Some(view_idx) = app.linear_dashboard_view_idx {
            selected_view = app.linear_dashboard_view_list[view_idx].clone();

            // TODO: This also needs to remove the ViewPanel
            // A populated view slot is selected
            if let Some(view) = selected_view {

                // Remove relevant ViewPanel from app.linear_dashboard_view_panel_list
                let view_panel_list_handle = app.linear_dashboard_view_panel_list.clone();
                let mut view_panel_list_lock = view_panel_list_handle.lock().unwrap();

                let filter_id = view["id"].clone();
                let filter_view_panel_exists = view_panel_list_lock
                    .iter()
                    .position(|e| { 
                        // debug!("filter_view_panel_exists comparing {:?} == {:?}", e.filter["id"], filter_id);   
                        e.filter["id"] == filter_id
                    });
                
                if let Some(filter_view_panel_idx) = filter_view_panel_exists {
                    view_panel_list_lock.remove(filter_view_panel_idx);
                }

                // Remove relevant view/filter JSON object
                app.linear_dashboard_view_list[view_idx] = None;

                // Sort app.linear_dashboard_view_list so that all Some's are first
                // e.g. ["View 1", "Empty Slot", "View 2", ...] -> [ "View 1", "View 2", "Empty Slot" ]
                app.linear_dashboard_view_list = app.linear_dashboard_view_list
                .iter()
                .filter_map(|e| {
                    e.as_ref().map(|_| e.clone())
                })
                .collect();

                while app.linear_dashboard_view_list.len() < 6 {
                    app.linear_dashboard_view_list.push(None);
                }
            }
        }
    }
}

pub async fn exec_select_view_panel_cmd(app: &mut App<'_>, view_panel_idx: usize) {

    // User is attempting to select a View Panel
    if Route::ActionSelect == app.route {
        // Verify that view_panel_idx is within bounds of app.linear_dashboard_view_panel_list.len()
        let view_panel_list_handle = app.linear_dashboard_view_panel_list.lock().unwrap();
        if view_panel_idx <= view_panel_list_handle.len() {

            // if so, update app.linear_dashboard_view_panel_selected to Some(view_panel_idx)
            app.linear_dashboard_view_panel_selected = Some(view_panel_idx);

            // If the DashboardViewPanel.issue_table_data is Some(Value::Array)
            // Verify Vec<Value>.len() > 0, and update app.view_panel_issue_selected to Some( table_state )
            let view_panel_handle = view_panel_list_handle[view_panel_idx-1].issue_table_data.lock().unwrap();

            if !view_panel_handle.is_empty() {
                let mut table_state = TableState::default();
                state_table::next(&mut table_state, &view_panel_handle);

                app.view_panel_issue_selected = Some( table_state );
            }

            // Unselect from app.actions
            app.actions.unselect();
        }
    }
}

pub fn exec_select_dashboard_view_list_cmd(app: &mut App) {
    app.linear_dashboard_view_list_selected = true;
    app.linear_custom_view_select.view_table_state = TableState::default();
}

pub fn exec_select_custom_view_select_cmd(app: &mut App) {
    app.linear_dashboard_view_list_selected = false;

    // If the CustomViewSelect.issue_table_data is Some(Value::Array)
    // Verify Vec<Value>.len() > 0, and update app.view_panel_issue_selected to Some( table_state )
    let custom_view_select_handle = app.linear_custom_view_select.view_table_data.lock().unwrap();

    if !custom_view_select_handle.is_empty() {
        let mut table_state = TableState::default();
        state_table::next(&mut table_state, &custom_view_select_handle);

        app.linear_custom_view_select.view_table_state = table_state;
    }
}


pub fn exec_open_linear_workflow_state_selection_cmd(app: &mut App, tx: &Sender<IOEvent>) {
    
    // Create pop-up on top of issue display component
    if Route::ActionSelect == app.route {
        debug!("draw_action_select - setting app.linear_draw_workflow_state_select to true");

        // Dispatch event to load workflow states for team of selected issue
        app.dispatch_event("load_workflows", tx);

        // Enable drawing of workflow state selection pop-up
        app.set_draw_issue_state_select(Platform::Linear, true);
    }
}

pub fn exec_move_back_cmd(app: &mut App, tx: &Sender<IOEvent>) {
    match app.route {

        // Unselect from List of Actions
        Route::ActionSelect => {

            // If state change cancelled, reset
            if app.linear_draw_workflow_state_select {
                app.linear_draw_workflow_state_select = false;
                app.linear_workflow_select = LinearWorkflowStateDisplayState::default();
            }

            // If a View Panel is selected, unselect it, reset app.linear_selected_issue_idx to None and
            // select app.actions()
            else if app.linear_dashboard_view_panel_selected.is_some() {
                app.linear_dashboard_view_panel_selected = None;
                app.linear_selected_issue_idx = None;
                app.actions.next();
            }
        },

        // Change Route to ActionSelect
        Route::DashboardViewDisplay => {
            // If the Custom View Select component is selected, don't change route
            if !app.linear_dashboard_view_list_selected {
                exec_select_dashboard_view_list_cmd(app);
            } else {
                app.change_route(Route::ActionSelect, &tx);
            }
        }
    }
}

pub async fn exec_confirm_cmd(app: &mut App<'_>, tx: &Sender<IOEvent>) {
    match app.route {
        Route::ActionSelect => {

            // If a state change is confirmed, dispatch & reset
            if app.linear_draw_workflow_state_select && app.linear_selected_workflow_state_idx.is_some() {
                app.dispatch_event("update_issue_workflow_state", &tx);
                app.linear_draw_workflow_state_select = false;
                app.linear_workflow_select = LinearWorkflowStateDisplayState::default();
            }

            else if let Some(i) = app.actions.state.selected() {
                match i {
                    0 => { app.change_route( Route::DashboardViewDisplay, &tx) },
                    _ => {}
                }
            }
        },
        // Select Custom View Select
        //     if already there: add Custom View to app.linear_dashboard_view_list if a view is selected
        Route::DashboardViewDisplay => {
            // Custom View Select component is not selected
            if app.linear_dashboard_view_list_selected {
                // Verify that a slot is selected
                // if so, switch to the CustomViewSelect Route to allow for selection of a Custom View
                if app.linear_dashboard_view_idx.is_some() {
                    exec_select_custom_view_select_cmd(app);
                }
            }
            // Custom View Select component is selected and a view from it is selected
            else if let Some(idx) = app.linear_selected_custom_view_idx  {
                // Add Custom View to app.linear_dashboard_view_list, if view selected

                // Custom View Select component is selected
                let custom_view_data_lock = app.linear_custom_view_select.view_table_data.lock().unwrap();

                info!("Got Custom View Data");
                let selected_view = custom_view_data_lock[idx].clone();

                // Attempt to add selected_view to first available slot in app.linear_dashboard_view_list
                // If no empty slots, do nothing

                info!("linear_dashboard_view_list: {:?}", app.linear_dashboard_view_list);

                /*
                let slot_idx_option = app.linear_dashboard_view_list
                                    .iter()
                                    .position(|x| match x {
                                        Some(_) => return true,
                                        None => return false,
                                    });
                */
                let slot_idx_option = app.linear_dashboard_view_idx;
                info!("slot_idx_option: {:?}", slot_idx_option);

                if let Some(slot_idx) = slot_idx_option {
                    info!("Updated linear_dashboard_view_list[{:?}] with selected_view: {:?}", slot_idx, selected_view);
                    app.linear_dashboard_view_list[slot_idx] = Some(selected_view);

                    // Sort app.linear_dashboard_view_list so that all Some's are first
                    app.linear_dashboard_view_list = app.linear_dashboard_view_list
                                                        .iter()
                                                        .filter_map(|e| { e.as_ref().map(|_| e.clone()) })
                                                        .collect();

                    while app.linear_dashboard_view_list.len() < 6 {
                        app.linear_dashboard_view_list.push(None);
                    }
                };

                drop(custom_view_data_lock);

                // Reset Selection back to Dashboard View List
                exec_select_dashboard_view_list_cmd(app);
            }
        }
    }
}

pub fn exec_scroll_down_cmd(app: &mut App, tx: &Sender<IOEvent>) {
    match app.route {
        // Select next Action
        Route::ActionSelect => {
            let mut load_paginated = false;

            // If the workflow state select panel is open, scroll down on modal
            if app.linear_draw_workflow_state_select {
                debug!("Attempting to scroll down on Workflow State Selection");
                let handle = &mut *app.linear_workflow_select.workflow_states_data.lock().unwrap();

                state_table::next(&mut app.linear_workflow_select.workflow_states_state, handle);
                app.linear_selected_workflow_state_idx = app.linear_workflow_select.workflow_states_state.selected();
            }
            // If a ViewPanel is selected, scroll down on the View Panel
            else if let Some(view_panel_selected_idx) = app.linear_dashboard_view_panel_selected {
                if let Some(table_state) = &app.view_panel_issue_selected {
                    let view_panel_list_handle = app.linear_dashboard_view_panel_list.lock().unwrap();

                    let view_panel_issue_handle = view_panel_list_handle[view_panel_selected_idx-1].issue_table_data.lock().unwrap();
                    let view_panel_loader_handle = view_panel_list_handle[view_panel_selected_idx-1].view_loader.lock().unwrap();

                    if !view_panel_issue_handle.is_empty() {
                        // Check if at end of app.view_panel_issue_selected
                        //  If true: Check if app.view_panel_list_handle[view_panel_selected_idx-1].view_loader.exhausted == false
                        //      If true: dispatch event to load next page view panel
                        //          and merge with current app.view_panel_list_handle[view_panel_selected_idx-1].issue_table_data

                        let is_last_element = state_table::is_last_element(table_state, &view_panel_issue_handle);
                        let loader_is_exhausted = if let Some(loader_val) = &*view_panel_loader_handle {
                                loader_val.exhausted
                            }
                            else {
                                false
                            };

                        if is_last_element && !loader_is_exhausted {
                            app.view_panel_to_paginate = view_panel_selected_idx-1;
                            load_paginated = true;
                        }
                        else {
                            app.view_panel_issue_selected = Some(state_table::with_next(table_state, &view_panel_issue_handle));
                        }
                    }
                }
            }
            // No View Panel selected or issue modal open, scroll on actions
            else {
                app.actions.next();
            }

            if load_paginated {
                app.dispatch_event("paginate_dashboard_view", &tx);
            }
        },

        // Select next Custom View Slot
        Route::DashboardViewDisplay => {
            if app.linear_dashboard_view_list_selected {
                state_table::next(&mut app.dashboard_view_display.view_table_state, &app.linear_dashboard_view_list);
                app.linear_dashboard_view_idx = app.dashboard_view_display.view_table_state.selected();
            }
            // Select next custom view from list of Linear custom views and update 'app.linear_selected_custom_view_idx'
            else {
                let mut load_paginated = false;
                {
                    let handle = &mut *app.linear_custom_view_select.view_table_data.lock().unwrap();

                    // if handle.len() == 0:
                    //     return; (either no custom views, or custom views being loaded)

                    // Check if at end of linear_custom_view_select.view_table_data
                    //  If true: Check if app.linear_custom_view_cursor.has_next_page = true
                    //      If true: dispatch event to load next page of linear issues
                    //          and merge with current linear_custom_view_select.view_table_data

                    if handle.is_empty() {
                        return;
                    }

                    // if called with len()=0, panics
                    let is_last_element = state_table::is_last_element(& app.linear_custom_view_select.view_table_state, handle);
                    let cursor_has_next_page;

                    {
                        let view_cursor_data_handle = app.linear_custom_view_cursor.lock().unwrap();
                        cursor_has_next_page = view_cursor_data_handle.has_next_page;
                    }

                    if is_last_element && cursor_has_next_page {
                        load_paginated = true;
                    }
                    else {
                        state_table::next(&mut app.linear_custom_view_select.view_table_state, handle);
                        app.linear_selected_custom_view_idx = app.linear_custom_view_select.view_table_state.selected();
                    }
                }
    
                if load_paginated {
                    app.dispatch_event("load_custom_views", &tx);
                }
            }
        }
    }
}

pub fn exec_scroll_up_cmd(app: &mut App) {

    match app.route {
        Route::ActionSelect => {

            // If the workflow state select panel is open, scroll down on modal
            if app.linear_draw_workflow_state_select {
                debug!("Attempting to scroll up on Workflow State Selection");
                let handle = &mut *app.linear_workflow_select.workflow_states_data.lock().unwrap();

                state_table::previous(&mut app.linear_workflow_select.workflow_states_state, handle);
                app.linear_selected_workflow_state_idx = app.linear_workflow_select.workflow_states_state.selected();
            }
            // If a ViewPanel is selected and no issue modal open, scroll down on the View Panel
            else if let Some(view_panel_selected_idx) = app.linear_dashboard_view_panel_selected {
                if let Some(table_state) = &app.view_panel_issue_selected {
                    let view_panel_list_handle = app.linear_dashboard_view_panel_list.lock().unwrap();
                    let view_panel_issue_handle = view_panel_list_handle[view_panel_selected_idx-1].issue_table_data.lock().unwrap();

                    if !view_panel_issue_handle.is_empty() {
                        app.view_panel_issue_selected = Some(state_table::with_previous(table_state, &view_panel_issue_handle));
                    }
                }
            }
            // No View Panel selected or issue modal open, scroll on actions
            else {
                app.actions.next();
            }
        },
        // Select previous Custom View Slot
        Route::DashboardViewDisplay => {
            if app.linear_dashboard_view_list_selected {
                state_table::previous(&mut app.dashboard_view_display.view_table_state, &app.linear_dashboard_view_list);
                app.linear_dashboard_view_idx = app.dashboard_view_display.view_table_state.selected();
            }
            else {
                let handle = &mut *app.linear_custom_view_select.view_table_data.lock().unwrap();

                // if handle.is_empty():
                //     return; (either no custom views, or custom views being loaded)
                if handle.is_empty() {
                    return;
                }

                // if called with len()=0, panics
                state_table::previous(&mut app.linear_custom_view_select.view_table_state, handle);
                app.linear_selected_custom_view_idx = app.linear_custom_view_select.view_table_state.selected();
            }
        }
    }
}