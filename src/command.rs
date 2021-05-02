
use termion::{event::Key,};

use std::collections::HashMap;

use crate::app::{App, Platform, Route};
use crate::network::IOEvent;
use crate::util::{ state_list, state_table };

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
    Add,
    Replace,
    Delete,
    SelectViewPanel(usize),


    OpenLinearWorkflowStateSelection,

}

/*
let tuples = vec![  ("q", Quit),
                    ("two", 2), 
                    ("three", 3)
                    ];

const str_to_cmd_map: HashMap<String, Command> = tuples.into_iter().collect();
*/

pub fn get_cmd(cmd_str: &mut String, input: Key) -> Option<Command> {
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
                // Add Command
                "a" => {
                    Some(Command::Add)
                },
                // Replace Command
                "r" => {
                    Some(Command::Replace)
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
                    Some(Command::SelectViewPanel(1))
                },
                "2" => {
                    Some(Command::SelectViewPanel(2))
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

pub async fn exec_add_cmd<'a>(app: &mut App<'a>, tx: &Sender<IOEvent>) {

    info!("Executing 'add' command");

    match app.route {
        // User is attempting to add a new Custom View to the Dashboard
        Route::DashboardViewDisplay => {
            // Verify that an empty slot is selected
            // if so, switch to the CustomViewSelect Route to allow for selection of a Custom View to add
            let mut view_is_selected = false;
            let mut selected_view: Option<Value> = None;
          
            if let Some(view_idx) = app.linear_dashboard_view_idx {
              view_is_selected = true;
              selected_view = app.linear_dashboard_view_list[view_idx].clone();

                if view_is_selected == true {
                    // An empty view slot is selected
                    if let None = selected_view {
                        app.change_route(Route::CustomViewSelect, &tx);
                    }
                }
            }
        },
        _ => {}
    }
}

pub async fn exec_replace_cmd<'a>(app: &mut App<'a>, tx: &Sender<IOEvent>) {
    info!("Executing 'replace' command");

    match app.route {
        // User is attempting to replace a Custom View with a new Custom View on the Dashboard
        Route::DashboardViewDisplay => {
            // Verify that a populated slot is selected
            // if so, switch to the CustomViewSelect Route to allow for selection of a Custom View to add
            let mut view_is_selected = false;
            let mut selected_view: Option<Value> = None;
            if let Some(view_idx) = app.linear_dashboard_view_idx {
              view_is_selected = true;
              selected_view = app.linear_dashboard_view_list[view_idx].clone();

                if view_is_selected == true {
                    // A populated view slot is selected
                    if let Some(_) = selected_view {
                        app.change_route(Route::CustomViewSelect, &tx);
                    }
                }
            }
        },
        _ => {}
    }
}

pub async fn exec_delete_cmd<'a>(app: &mut App<'a>, tx: &Sender<IOEvent>) {
    info!("Executing 'delete' command");
    match app.route {
        // User is attempting to remove a Custom View on the Dashboard
        Route::DashboardViewDisplay => {
            // Verify that a populated slot is selected
            // if so, set it to None
            let mut view_is_selected = false;
            let mut selected_view: Option<Value> = None;
            if let Some(view_idx) = app.linear_dashboard_view_idx {
              view_is_selected = true;
              selected_view = app.linear_dashboard_view_list[view_idx].clone();

                if view_is_selected == true {
                    // A populated view slot is selected
                    if let Some(_) = selected_view {
                        app.linear_dashboard_view_list[view_idx] = None;

                        // Sort app.linear_dashboard_view_list so that all Some's are first
                        app.linear_dashboard_view_list = app.linear_dashboard_view_list
                        .iter()
                        .filter_map(|e| {
                            match e {
                                Some(_) => Some(e.clone()),
                                None => None,
                            }
                        })
                        .collect();
                        while app.linear_dashboard_view_list.len() < 6 {
                            app.linear_dashboard_view_list.push(None);
                        }
                    }
                }
            }
        },
        _ => {}
    }
}

pub async fn exec_select_view_panel_cmd<'a>(app: &mut App<'a>, view_panel_idx: usize, tx: &Sender<IOEvent>) {
    match app.route {
        // User is attempting to select a View Panel
        Route::ActionSelect => {
            // Verify that view_panel_idx is within bounds of app.linear_dashboard_view_panel_list.len()
            let view_panel_list_handle = app.linear_dashboard_view_panel_list.lock().unwrap();
            if view_panel_idx <= view_panel_list_handle.len() {

                // if so, update app.linear_dashboard_view_panel_selected to Some(view_panel_idx)
                app.linear_dashboard_view_panel_selected = Some(view_panel_idx);

                // If the DashboardViewPanel.issue_table_data is Some(Value::Array)
                // Verify Vec<Value>.len() > 0, and update app.view_panel_issue_selected to Some( table_state )
                let view_panel_handle = view_panel_list_handle[view_panel_idx-1].issue_table_data.lock().unwrap();

                if let Some(issue_data) = &*view_panel_handle {
                    if let Value::Array(issue_vec) = issue_data {
                        if issue_vec.len() > 0 {
                            let mut table_state = TableState::default();
                            state_table::next(&mut table_state, &issue_vec);

                            app.view_panel_issue_selected = Some( table_state );
                        }
                    }
                }

                // Unselect from app.actions
                app.actions.unselect();
            }
        },

        _ => {}
    }
}


pub fn exec_open_linear_workflow_state_selection_cmd(app: &mut App, tx: &Sender<IOEvent>) {
    match app.route {
        // Create pop-up on top of issue display component
        Route::LinearInterface => {
            // Dispatch event to begin loading new data
            app.dispatch_event("load_workflows", tx);

            // Enable drawing of workflow state selection pop-up
            app.set_draw_issue_state_select(Platform::Linear, true);
        }
        _ => {},
    }
}

pub fn exec_move_back_cmd(app: &mut App, tx: &Sender<IOEvent>) {
    match app.route {

        // Unselect from List of Actions
        Route::ActionSelect => {
            // If a View Panel is selected, unselect it, reset app.linear_selected_issue_idx to None and
            // select app.actions()
            if let Some(_) = app.linear_dashboard_view_panel_selected {
                app.linear_dashboard_view_panel_selected = None;
                app.linear_selected_issue_idx = None;
                app.actions.next();
            }
        },

        // Change Route to ActionSelect
        Route::DashboardViewDisplay => {
            app.change_route(Route::ActionSelect, &tx);
        }

        // Unselect from Selection of Teams
        Route::TeamSelect => {
            state_list::unselect(&mut app.linear_team_select.teams_state);
        },

        // Unselect from list of Linear Issues
        Route::LinearInterface => {
            state_table::unselect(&mut app.linear_issue_display.issue_table_state);
        }

        _ => {}
    }
}

pub async fn exec_confirm_cmd<'a>(app: &mut App<'a>, tx: &Sender<IOEvent>) {
    match app.route {
        Route::ActionSelect => match app.actions.state.selected() {
            Some(i) => {
                match i {
                    0 => { app.change_route( Route::DashboardViewDisplay, &tx) },
                    1 => { app.change_route( Route::TeamSelect, &tx) }
                    _ => {}
                }
            }
            _ => {}
        },
        // Add Custom View to app.linear_dashboard_view_list if a view is selected
        Route::CustomViewSelect => match app.linear_selected_custom_view_idx {
            // Add Custom View to app.linear_dashboard_view_list
            Some(idx) => {
                let custom_view_data_lock = app.linear_custom_view_select.view_table_data.lock().unwrap();

                match &*custom_view_data_lock {
                    Some(view_data) => {
                        info!("Got Custom View Data");
                        let selected_view = view_data[idx].clone();

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
                        
                        match slot_idx_option {
                            Some(slot_idx) => {
                                info!("Updated linear_dashboard_view_list[{:?}] with selected_view: {:?}", slot_idx, selected_view);
                                app.linear_dashboard_view_list[slot_idx] = Some(selected_view);

                                // Sort app.linear_dashboard_view_list so that all Some's are first
                                app.linear_dashboard_view_list = app.linear_dashboard_view_list
                                                                    .iter()
                                                                    .filter_map(|e| {
                                                                        match e {
                                                                            Some(_) => Some(e.clone()),
                                                                            None => None,
                                                                        }
                                                                    })
                                                                    .collect();
                                while app.linear_dashboard_view_list.len() < 6 {
                                    app.linear_dashboard_view_list.push(None);
                                }

                            },
                            None => {},
                        };
                    },
                    None => {}
                };
                drop(custom_view_data_lock);
                // Change Route to Route::DashboardViewDisplay
                app.change_route( Route::DashboardViewDisplay, &tx);
            },
            None => {},
        }

        // Switch Route as long as a team is selected
        Route::TeamSelect => match app.linear_selected_team_idx {
            Some(_) => { app.change_route(Route::LinearInterface, &tx) },
            None => {},
        },
        // Dispatch Update Issue Workflow State command if User selects a workflow state for a given Issue
        Route::LinearInterface => {
            app.dispatch_event("update_issue_workflow", &tx);
            // Close Workflow States Panel
            app.set_draw_issue_state_select(Platform::Linear, false);

        },
        _ => {}
    }
}

pub fn exec_scroll_down_cmd(app: &mut App, tx: &Sender<IOEvent>) {
    match app.route {
        // Select next Action
        Route::ActionSelect => {
            let mut load_paginated = false;
            // If a ViewPanel is selected, scroll down on the View Panel
            if let Some(view_panel_selected_idx) = app.linear_dashboard_view_panel_selected {
                if let Some(table_state) = &app.view_panel_issue_selected {
                    let view_panel_list_handle = app.linear_dashboard_view_panel_list.lock().unwrap();

                    let view_panel_issue_handle = view_panel_list_handle[view_panel_selected_idx-1].issue_table_data.lock().unwrap();
                    let view_panel_loader_handle = view_panel_list_handle[view_panel_selected_idx-1].view_loader.lock().unwrap();
                    if let Some(view_panel_issue_data) = &*view_panel_issue_handle {
                        if let Value::Array(view_panel_issue_vec) = view_panel_issue_data {

                            // Check if at end of app.view_panel_issue_selected
                            //  If true: Check if app.view_panel_list_handle[view_panel_selected_idx-1].view_loader.exhausted == false
                            //      If true: dispatch event to load next page view panel
                            //          and merge with current app.view_panel_list_handle[view_panel_selected_idx-1].issue_table_data

                            let is_last_element = state_table::is_last_element(table_state, view_panel_issue_vec);
                            let loader_is_exhausted = if let Some(loader_val) = &*view_panel_loader_handle {
                                    loader_val.exhausted
                                }
                                else {
                                    false
                                };

                            if is_last_element == true && loader_is_exhausted == false {
                                app.view_panel_to_paginate = view_panel_selected_idx-1;
                                load_paginated = true;
                            }
                            else {
                                app.view_panel_issue_selected = Some(state_table::with_next(table_state, &view_panel_issue_vec));
                            }

                        }
                    }
                }
            }
            // No View Panel selected, scroll on actions
            else {
                app.actions.next();
            }

            if load_paginated == true {
                app.dispatch_event("paginate_dashboard_view", &tx);
            }
        },

        // Select next Custom View Slot
        Route::DashboardViewDisplay => {
            state_table::next(&mut app.dashboard_view_display.view_table_state, &app.linear_dashboard_view_list);
            app.linear_dashboard_view_idx = app.dashboard_view_display.view_table_state.selected();
        },

        // Select next custom view from list of Linear custom views and update 'app.linear__selected_custom_view_idx'
        Route::CustomViewSelect => {
            let mut load_paginated = false;
            {
                let handle = &mut *app.linear_custom_view_select.view_table_data.lock().unwrap();
                match *handle {
                    Some(ref mut x) => {
                        match x.as_array() {
                            Some(y) => {
                                // Check if at end of linear_custom_view_select.view_table_data
                                //  If true: Check if app.linear_custom_view_cursor.has_next_page = true
                                //      If true: dispatch event to load next page of linear issues
                                //          and merge with current linear_custom_view_select.view_table_data

                                let is_last_element = state_table::is_last_element(& app.linear_custom_view_select.view_table_state, y);
                                let mut cursor_has_next_page = false;
                                {
                                    let view_cursor_data_handle = app.linear_custom_view_cursor.lock().unwrap();
                                    cursor_has_next_page = view_cursor_data_handle.has_next_page;
                                }

                                if is_last_element == true && cursor_has_next_page == true {
                                    load_paginated = true;
                                }
                                else {
                                    state_table::next(&mut app.linear_custom_view_select.view_table_state, y);
                                    app.linear_selected_custom_view_idx = app.linear_custom_view_select.view_table_state.selected();
                                    info!("app.linear_selected_custom_view_idx: {:?}", app.linear_selected_custom_view_idx);
                                }
                            },
                            None => {},
                        }
                    }
                    _ => {},
                }
            }

            if load_paginated == true {
                app.dispatch_event("load_custom_views", &tx);
            }
        },

        // Select next team from list of Linear teams and update 'app.linear_selected_team_idx'
        Route::TeamSelect => {
            let handle = &mut *app.linear_team_select.teams_data.lock().unwrap();
            match *handle {
                Some(ref mut x) => {
                    match x.as_array() {
                        Some(y) => {
                            state_list::next(&mut app.linear_team_select.teams_state, y);
                            app.linear_selected_team_idx = app.linear_team_select.teams_state.selected();
                        },
                        None => {},
                    }
                }
                _ => {},
            }
        },

        // Select next issue from list of Linear issues and update 'app.linear_selected_issue_idx'
        Route::LinearInterface => {
            // If User is not selecting a new workflow state for an issue, select next issue
            let mut load_paginated = false;
            if *app.draw_issue_state_select(Platform::Linear) == false {
                {
                    let handle = &mut *app.linear_issue_display.issue_table_data.lock().unwrap();
                    match *handle {
                        Some(ref mut x) => {
                            match x.as_array() {
                                Some(y) => {
                                    // Check if at end of linear_issue_display.issue_table_state
                                    //  If true: Check if app.linear_issue_cursor.has_next_page = true
                                    //      If true: dispatch event to load next page of linear issues
                                    //          and merge with current linear_issue_display.issue_table_state

                                    let is_last_element = state_table::is_last_element(& app.linear_issue_display.issue_table_state, y);
                                    let mut cursor_has_next_page = false;
                                    {
                                        let issue_cursor_data_handle = app.linear_issue_cursor.lock().unwrap();
                                        cursor_has_next_page = issue_cursor_data_handle.has_next_page;
                                    }

                                    if is_last_element == true && cursor_has_next_page == true {
                                        load_paginated = true;
                                    }
                                    else {
                                        state_table::next(&mut app.linear_issue_display.issue_table_state, y);
                                        app.linear_selected_issue_idx = app.linear_issue_display.issue_table_state.selected();
                                        info!("app.linear_selected_issue_idx: {:?}", app.linear_selected_issue_idx);
                                    }
                                },
                                None => {},
                            }
                        }
                        _ => {},
                    }
                }

                if load_paginated == true {
                    app.dispatch_event("load_issues_paginated", &tx);
                }
            }
            // If User is selecting a new workflow state for an issue, select next workflow state
            else {
                info!("Attempting to scroll down on Workflow State Selection");
                let handle = &mut *app.linear_workflow_select.workflow_states_data.lock().unwrap();
                match *handle {
                    Some(ref mut x) => {
                        match x.as_array() {
                            Some(y) => {
                                state_table::next(&mut app.linear_workflow_select.workflow_states_state, y);
                                app.linear_selected_workflow_state_idx = app.linear_workflow_select.workflow_states_state.selected();
                                // info!("app.linear_selected_workflow_state_idx: {:?}", app.linear_selected_workflow_state_idx);
                            },
                            None => {},
                        }
                    },
                    None => {}
                }
            }
        }
    }
}

pub fn exec_scroll_up_cmd(app: &mut App) {

    match app.route {
        Route::ActionSelect => {
            // If a ViewPanel is selected, scroll down on the View Panel
            if let Some(view_panel_selected_idx) = app.linear_dashboard_view_panel_selected {
                if let Some(table_state) = &app.view_panel_issue_selected {
                    let view_panel_list_handle = app.linear_dashboard_view_panel_list.lock().unwrap();
                    let view_panel_issue_handle = view_panel_list_handle[view_panel_selected_idx-1].issue_table_data.lock().unwrap();
                    if let Some(view_panel_issue_data) = &*view_panel_issue_handle {
                        if let Value::Array(view_panel_issue_vec) = view_panel_issue_data {
                            app.view_panel_issue_selected = Some(state_table::with_previous(table_state, &view_panel_issue_vec));
                        }
                    }
                }
            }
            // No View Panel selected, scroll on actions
            else {
                app.actions.next();
            }
        },
        // Select previous Custom View Slot
        Route::DashboardViewDisplay => {
            state_table::previous(&mut app.dashboard_view_display.view_table_state, &app.linear_dashboard_view_list);
            app.linear_dashboard_view_idx = app.dashboard_view_display.view_table_state.selected();
        },
        Route::CustomViewSelect => {
            let handle = &mut *app.linear_custom_view_select.view_table_data.lock().unwrap();
            match *handle {
                Some(ref mut x) => {
                    match x.as_array() {
                        Some(y) => {
                            state_table::previous(&mut app.linear_custom_view_select.view_table_state, y);
                            app.linear_selected_issue_idx = app.linear_custom_view_select.view_table_state.selected();
                            info!("app.linear_selected_custom_view_idx: {:?}", app.linear_selected_custom_view_idx);
                        },
                        None => {},
                    }
                }
                _ => {},
            }
        }
        Route::TeamSelect => {
            let handle = &mut *app.linear_team_select.teams_data.lock().unwrap();
            match handle {
                Some(ref mut x) => {
                    match x.as_array() {
                        Some(y) => {
                            state_list::previous(&mut app.linear_team_select.teams_state, y);
                            app.linear_selected_team_idx = app.linear_team_select.teams_state.selected();
                        },
                        None => {},
                    }
                },
                _ => {},
            }
        },
        Route::LinearInterface => {
            // If User is not selecting a new workflow state for an issue, select previous issue
            if *app.draw_issue_state_select(Platform::Linear) == false {
                let handle = &mut *app.linear_issue_display.issue_table_data.lock().unwrap();
                match *handle {
                    Some(ref mut x) => {
                        match x.as_array() {
                            Some(y) => {
                                state_table::previous(&mut app.linear_issue_display.issue_table_state, y);
                                app.linear_selected_issue_idx = app.linear_issue_display.issue_table_state.selected();
                                info!("app.linear_selected_issue_idx: {:?}", app.linear_selected_issue_idx);
                            },
                            None => {},
                        }
                    }
                    _ => {},
                }
            }
            // If User is selecting a new workflow state for an issue, select previous workflow state
            else {
                info!("Attempting to scroll up on Workflow State Selection");
                let handle = &mut *app.linear_workflow_select.workflow_states_data.lock().unwrap();
                match *handle {
                    Some(ref mut x) => {
                        match x.as_array() {
                            Some(y) => {
                                state_table::previous(&mut app.linear_workflow_select.workflow_states_state, y);
                                app.linear_selected_workflow_state_idx = app.linear_workflow_select.workflow_states_state.selected();
                            },
                            None => {},
                        }
                    },
                    None => {}
                }
            }
        }
    }
}