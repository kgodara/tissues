
use termion::{event::Key,};

use std::collections::HashMap;

use crate::app::{App, Platform, Route};
use crate::network::IOEvent;
use crate::util::{ state_list, state_table };

use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub enum Command {

    // Arow Key Commands
    MoveBack,
    ScrollDown,
    ScrollUp,
    Confirm,

    // Char Commands
    Quit,
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
                // Modify Command
                "m" => {
                    Some(Command::OpenLinearWorkflowStateSelection)
                }
                _ => {
                    None
                }
            }
        },

        _ => { None }
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

pub fn exec_move_back_cmd(app: &mut App) {
    match app.route {

        // Unselect from List of Actions
        Route::ActionSelect => app.actions.unselect(),

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
                    0 => { app.change_route( Route::CustomViewSelect, &tx).await },
                    1 => { app.change_route( Route::TeamSelect, &tx).await }
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
                        let selected_view = view_data[idx].clone();
                        app.linear_dashboard_view_list.push(selected_view);
                    },
                    None => {}
                };
                drop(custom_view_data_lock);
                // TEMP: Dispatch "load_view_issues" Command
                app.dispatch_event("load_view_issues", &tx);
            },
            None => {},
        }

        // Switch Route as long as a team is selected
        Route::TeamSelect => match app.linear_selected_team_idx {
            Some(_) => { app.change_route(Route::LinearInterface, &tx).await },
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
        Route::ActionSelect => app.actions.next(),

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
        _ => {}
    }
}

pub fn exec_scroll_up_cmd(app: &mut App) {

    match app.route {
        Route::ActionSelect => app.actions.previous(),
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
        _ => {}
    }

}