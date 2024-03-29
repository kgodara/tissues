
// use termion::{event::Key,};
use crossterm::event::KeyCode;
use unicode_segmentation::UnicodeSegmentation;

use std::sync::atomic::{ Ordering };

use crate::app::{App, Platform, AppEvent, Route, InputMode};
use crate::util::{
    table_state,
    dashboard::{ fetch_selected_view_panel_issue, fetch_selected_view_panel_idx, },
    event_crossterm::{ Events },
};

use crate::linear::{
    config::LinearConfig,
    schema::CustomView,
    client::{ IssueFieldObject }
};

use crate::constants::{
    LINEAR_TOKEN_LEN,
    IssueModificationOp
};

use crate::components::{ InputComponent, token_entry::{ TokenValidationState }, title_entry::{ TitleValidationState } };

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

    // User Input related Commands
    
    EditorEnter,
    EditorInput(char),

    EditorMoveBackward,
    EditorMoveForward,

    EditorDelete,

    EditorSubmit,
    EditorExit,

    // Char Commands
    Quit,
    Delete,
    SelectViewPanel(usize),

    RefreshViewPanel,
    ExpandIssue,

    SelectDashboardViewList,
    SelectCustomViewSelect,

    OpenIssueOpInterface(IssueModificationOp),

    // OpenLinearWorkflowStateSelection,
    // OpenLinearAssigneeSelection,

}


pub fn get_cmd(cmd_str: &mut String, input: KeyCode, current_route: &Route, input_mode: &InputMode) -> Option<Command> {

    // Editor input/submit/exit commands
    if *input_mode == InputMode::Edit {
        return match input {
            KeyCode::Esc => Some(Command::EditorExit),
            KeyCode::Char('\n') => Some(Command::EditorSubmit),

            // windows support
            KeyCode::Enter => Some(Command::EditorSubmit),

            KeyCode::Right => Some(Command::EditorMoveForward),
            KeyCode::Left => Some(Command::EditorMoveBackward),

            KeyCode::Char(c) => Some(Command::EditorInput(c)),
            KeyCode::Backspace => Some(Command::EditorDelete),
            _ => {
                debug!("unsupported editor KeyCode: {:?}", input);
                None
            }
        }
    }

    match input {
        // Navigation/Confirmation related inputs
        // These will always clear the command string
        KeyCode::Left => {
            Some(Command::MoveBack)
        },
        KeyCode::Down => {
            Some(Command::ScrollDown)
        },
        KeyCode::Up => {
            Some(Command::ScrollUp)
        },
        KeyCode::Right => {
            Some(Command::Confirm)
        },

        // Contextual User commands
        KeyCode::Char(ch) => {
            cmd_str.push(ch);
            match cmd_str.as_str() {
                // Quit Command
                "q" => {
                    Some(Command::Quit)
                },
                // EditorEnter Command
                "e" => {
                    Some(Command::EditorEnter)
                },
                // Delete Command
                "d" => {
                    Some(Command::Delete)
                },
                // Refresh Command
                "r" => {
                    Some(Command::RefreshViewPanel)
                },
                "f" => {
                    Some(Command::ExpandIssue)  
                },
                // Modify Command
                "t" => {
                    Some(Command::OpenIssueOpInterface(IssueModificationOp::Title))
                },
                "w" => {
                    Some(Command::OpenIssueOpInterface(IssueModificationOp::WorkflowState))
                },
                "a" => {
                    Some(Command::OpenIssueOpInterface(IssueModificationOp::Assignee))
                },
                "p" => {
                    Some(Command::OpenIssueOpInterface(IssueModificationOp::Project))
                },
                "c" => {
                    Some(Command::OpenIssueOpInterface(IssueModificationOp::Cycle))
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
        KeyCode::Esc => {
            Some(Command::EditorExit)
        }

        _ => { None }
    }
}


pub fn exec_editor_focus_cmd(app: &mut App<'_>, events: &mut Events) {
        events.disable_exit_key();
        app.input_mode = InputMode::Edit;
}

pub fn exec_editor_input_cmd(app: &mut App<'_>, ch: &char) {
    // Verify user is entering access token
    match app.active_input {
        InputComponent::TokenEntry => { app.token_entry.input.insert(*ch) },
        InputComponent::TitleEntry => { app.title_entry.input.insert(*ch) },
    }
}

pub fn exec_editor_move_forward_cmd(app: &mut App<'_>) {
    match app.active_input {
        InputComponent::TokenEntry => { app.token_entry.input.move_cursor_forwards() },
        InputComponent::TitleEntry => { app.title_entry.input.move_cursor_forwards() },
    }
}

pub fn exec_editor_move_back_cmd(app: &mut App<'_>) {
    match app.active_input {
        InputComponent::TokenEntry => { app.token_entry.input.move_cursor_back() },
        InputComponent::TitleEntry => { app.title_entry.input.move_cursor_back() },
    }
}

pub fn exec_editor_delete_cmd(app: &mut App<'_>) {
    match app.active_input {
        InputComponent::TokenEntry => { app.token_entry.input.delete() },
        InputComponent::TitleEntry => { app.title_entry.input.delete() },
    }
}

pub fn exec_editor_submit_cmd(app: &mut App<'_>, events: &mut Events) {

    events.enable_exit_key();

    if app.input_mode == InputMode::Edit {
        match app.active_input {
            InputComponent::TokenEntry => {
                let submission_len: u16 = unicode_width::UnicodeWidthStr::width(app.token_entry.input.input.as_str()) as u16;
                // Verify length is satisfactory for linear access token
                info!("exec_editor_submit_cmd() - {:?} == {:?}", submission_len, LINEAR_TOKEN_LEN);
                if submission_len == LINEAR_TOKEN_LEN {
                    app.dispatch_event(AppEvent::LoadViewer);
                }
                else {
                    let mut token_validation_state_lock = app.token_entry.token_validation_state.lock().unwrap();
                    *token_validation_state_lock = TokenValidationState::Invalid;
                }
            },
            InputComponent::TitleEntry => {
                // Execute length check, validation
                //      if invalid: set invalid message, etc
                if app.title_entry.input.input.graphemes(true).count() > 0 {
                    info!("exec_editor_submit_cmd - dispatching 'update_issue' event");
                    app.dispatch_event(AppEvent::UpdateIssue);
                    app.modifying_issue = false;
                } else {
                    let mut title_validation_state_lock = app.title_entry.title_validation_state.lock().unwrap();
                    *title_validation_state_lock = TitleValidationState::Invalid;
                }
            }
        }
    }

    app.input_mode = InputMode::Normal;
}

pub fn exec_editor_exit_cmd(app: &mut App<'_>, events: &mut Events) {
    events.enable_exit_key();
    app.input_mode = InputMode::Normal;

    // If editing the title, close the modal as well
    if app.modifying_issue {
        app.input_mode = InputMode::Normal;
        app.modifying_issue = false;
        app.linear_issue_op_interface.reset_op();
    }
}


pub async fn exec_delete_cmd(app: &mut App<'_>) {
    info!("Executing 'delete' command");

    let cur_route: Route = (*app.route.lock().unwrap()).clone();

    // User is attempting to remove a Custom View on the Dashboard
    if Route::DashboardViewDisplay == cur_route {
        // Verify that a populated slot is selected
        // if so, set it to None
        let selected_view: Option<CustomView>;
        if let Some(view_idx) = app.linear_dashboard_view_idx {
            selected_view = app.linear_dashboard_view_list[view_idx].clone();

            // TODO: This also needs to remove the ViewPanel
            // A populated view slot is selected
            if let Some(view) = selected_view {

                // Remove relevant ViewPanel from app.linear_dashboard_view_panel_list
                let view_panel_list_handle = app.linear_dashboard_view_panel_list.clone();
                let mut view_panel_list_lock = view_panel_list_handle.lock().unwrap();

                let filter_id = view.id.clone();
                let filter_view_panel_exists = view_panel_list_lock
                    .iter()
                    .position(|e| {
                        // debug!("filter_view_panel_exists comparing {:?} == {:?}", e.filter["id"], filter_id);   
                        e.view.id == filter_id
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

                // Serialize new Custom View List
                LinearConfig::save_view_list(app.linear_dashboard_view_list.clone());
            }
        }
    }
}

pub fn exec_select_view_panel_cmd(app: &mut App<'_>, view_panel_idx: usize) {

    let cur_route: Route = app.route.lock().unwrap().clone();    

    // User is attempting to select a View Panel
    if Route::ActionSelect == cur_route {
        // Verify that view_panel_idx is within bounds of app.linear_dashboard_view_panel_list.len()
        // &&
        // Verify issue modification not in progress
        let view_panel_list_handle = app.linear_dashboard_view_panel_list.lock().unwrap();
        if view_panel_idx <= view_panel_list_handle.len() && !app.modifying_issue {

            // if so, update app.linear_dashboard_view_panel_selected to Some(view_panel_idx)
            app.linear_dashboard_view_panel_selected = Some(view_panel_idx);

            // If the DashboardViewPanel.issue_table_data is Some(Value::Array)
            // Verify Vec<Value>.len() > 0, and update app.view_panel_issue_selected to Some( table_state )
            let view_panel_handle = view_panel_list_handle[view_panel_idx-1].issue_table_data.lock().unwrap();

            // select initial issue in newly selected view panel
            if !view_panel_handle.is_empty() {
                let mut table_state = TableState::default();
                table_state::next(&mut table_state, &view_panel_handle);

                app.view_panel_issue_selected = Some( table_state );
            }

            drop(view_panel_handle);
            drop(view_panel_list_handle);
            
            // updated expanded issue
            if app.issue_to_expand.is_some() {
                exec_expand_issue_cmd(app);
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
        table_state::next(&mut table_state, &custom_view_select_handle);

        app.linear_selected_custom_view_idx = table_state.selected();
        app.linear_custom_view_select.view_table_state = table_state;
    }
}

pub fn exec_refresh_view_panel_cmd(app: &mut App) {
    // Execute command if:
    //     view panel is selected &&
    //     view panel is not loading
    //     expanded issue modal not open

    if let Some(idx) = fetch_selected_view_panel_idx(app) {
        let view_panel_list_lock = app.linear_dashboard_view_panel_list.lock().unwrap();


        // let mut loading_init_lock = view_panel_list_handle[self.view_panel_to_paginate].loading.lock().unwrap();
        // let mut panel_loading_lock = view_panel_list_lock[idx].loading.lock().unwrap();

        debug!("idx: {:?}", idx);
        let is_panel_loading = &view_panel_list_lock[idx].loading;


        if !is_panel_loading.load(Ordering::Relaxed) && app.issue_to_expand.is_none() {

            // Reset visual selection
            app.view_panel_issue_selected = Some(TableState::default());

            // Reset the following view panel fields before dispatching event: "paginate_dashboard_view"
            //     pub issue_table_data: Arc<Mutex<Vec<Value>>>,
            //     pub view_loader: Arc<Mutex<Option<ViewLoader>>>,
            //     pub loading: Arc<AtomicBool>,

            let mut cursor_lock = view_panel_list_lock[idx].view_cursor.lock().unwrap();
            let mut panel_issue_lock = view_panel_list_lock[idx].issue_table_data.lock().unwrap();

            *panel_issue_lock = vec![];
            *cursor_lock = None;

            is_panel_loading.store(false, Ordering::Relaxed);

            drop(cursor_lock);
            drop(panel_issue_lock);

            drop(view_panel_list_lock);

            // mark panel for pagination
            app.view_panel_to_paginate = idx;

            app.dispatch_event(AppEvent::PaginateDashboardView);

        }
    }
}

pub fn exec_expand_issue_cmd(app: &mut App) {
    // Execute command if:
    //     view panel issue is selected

    if let Some(issue_obj) = fetch_selected_view_panel_issue(app) {
        app.issue_to_expand = Some(issue_obj.clone());
    } else {
        app.issue_to_expand = None;
    }
}


// Issue Modification Commands

pub fn exec_open_issue_op_interface_cmd(app: &mut App, op: IssueModificationOp) {

    let cur_route: Route = app.route.lock().unwrap().clone();

    if Route::ActionSelect == cur_route {

        // If matching op interface modal is open, close it
        if app.linear_issue_op_interface.current_op == Some(op) {
            exec_move_back_cmd(app);
        }

        // Enable drawing of issue op interface if:
        //     expanded issue modal not open
        else if app.issue_to_expand.is_none() {
            app.linear_issue_op_interface.current_op = Some(op);
            app.modifying_issue = true;

            // If IssueModificationOp::Title,
            // set app.title_entry.input to issue title
            // 
            if op == IssueModificationOp::Title {
                let issue_option = fetch_selected_view_panel_issue(app);
                if let Some(issue_obj) = issue_option {
                    app.title_entry.input.set_input(issue_obj.title.to_string());
                    
                    app.input_mode = InputMode::Edit;
                    app.active_input = InputComponent::TitleEntry;
                }
            }

            app.dispatch_event(AppEvent::LoadIssueOpData);
        }
    }
}



pub fn exec_move_back_cmd(app: &mut App) {

    let cur_route: Route = app.route.lock().unwrap().clone();

    match cur_route {

        Route::ConfigInterface => {
            // nowhere to move back to
        },

        // Unselect from List of Actions
        Route::ActionSelect => {

            // If state change cancelled, reset
            if app.modifying_issue {
                app.modifying_issue = false;
                app.linear_issue_op_interface.reset_op();

                // disable editor, only relevant if op was title modification
                app.input_mode = InputMode::Normal;
            }

            // If expanded Issue view is open, close modal
            else if app.issue_to_expand.is_some() {
                app.issue_to_expand = None;
            }

            // If a View Panel is selected, unselect it, reset app.linear_dashboard_view_panel_selected to None and
            // select app.actions()
            else if app.linear_dashboard_view_panel_selected.is_some() {
                app.linear_dashboard_view_panel_selected = None;
                app.actions.next();
            }
        },

        // Change Route to ActionSelect
        Route::DashboardViewDisplay => {
            // If the Custom View Select component is selected, don't change route
            if !app.linear_dashboard_view_list_selected {
                exec_select_dashboard_view_list_cmd(app);
            } else {
                app.change_route(Route::ActionSelect);
            }
        }
    }
}

pub async fn exec_confirm_cmd(app: &mut App<'_>) {

    let cur_route: Route = app.route.lock().unwrap().clone();

    match cur_route {
        // Unlike exec_editor_submit_cmd(), this does not submit a new access token,
        Route::ConfigInterface => {},
        Route::ActionSelect => {

            let valid_selection: bool = app.linear_issue_op_interface.is_valid_selection_for_update(&app.title_entry.input.input);

            // If a state change is confirmed, dispatch & reset
            if app.modifying_issue && valid_selection {
                app.dispatch_event(AppEvent::UpdateIssue);
                app.modifying_issue = false;
            }
            // If user has chosen the "Modify Dashboard" action
            // only allow if timezone load is complete
            else if let Some(i) = app.actions.state.selected() {
                if i == 0 {
                    app.change_route( Route::DashboardViewDisplay)
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

                // Attempt to add selected_view to selected slot in app.linear_dashboard_view_list
                // sort after adding, so all filled slots are first

                info!("linear_dashboard_view_list: {:?}", app.linear_dashboard_view_list);

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

                    // Serialize new Custom View List
                    LinearConfig::save_view_list(app.linear_dashboard_view_list.clone());
                };

                drop(custom_view_data_lock);

                // Reset Selection back to Dashboard View List
                exec_select_dashboard_view_list_cmd(app);
            }
        }
    }
}

pub fn exec_scroll_down_cmd(app: &mut App) {

    let cur_route: Route = app.route.lock().unwrap().clone();

    match cur_route {

        Route::ConfigInterface => {
            // nowhere to scroll
        },

        // Select next Action
        Route::ActionSelect => {
            let mut load_paginated = false;

            // Don't scroll down if entering issue title
            if app.modifying_issue && app.linear_issue_op_interface.current_op == Some(IssueModificationOp::Title) {
                return;
            }
            // If the issue op interface is open, scroll down on modal
            else if app.modifying_issue {

                debug!("Attempting to scroll down on IssueOpInterface");




                let mut load_paginated = false;
                {
                    // if handle.len() == 0:
                    //     return; (either no issue relatable objects, or being loaded)
                    let issue_op_obj_vec: Vec<IssueFieldObject>;

                    if let Some(result) = app.linear_issue_op_interface.table_data_from_op() {
                        issue_op_obj_vec = result;
                    } else {
                        return;
                    }

                    // Check if at end of linear_issue_op_interface.table_data_from_op()
                    //  If true: Check if app.linear_issue_op_interface.cursor.has_next_page = true
                    //      If true: dispatch event to load next page of linear issues
                    //          and merge with current linear_custom_view_select.view_table_data

                    if issue_op_obj_vec.is_empty() {
                        return;
                    }

                    // if called with len()=0, panics
                    let is_last_element = table_state::is_last_element(& app.linear_issue_op_interface.data_state, &issue_op_obj_vec);
                    let cursor_has_next_page;

                    {
                        let issue_op_cursor_data_lock = app.linear_issue_op_interface.cursor.lock().unwrap();
                        cursor_has_next_page = issue_op_cursor_data_lock.has_next_page;
                    }

                    debug!("exec_scroll_down_cmd::Route::ActionSelect - is_last_element, cursor_has_next_page: {:?}, {:?}", is_last_element, cursor_has_next_page);

                    if is_last_element && cursor_has_next_page {
                        load_paginated = true;
                    }
                    else {
                        table_state::next(&mut app.linear_issue_op_interface.data_state, &issue_op_obj_vec);
                        app.linear_issue_op_interface.selected_idx = app.linear_issue_op_interface.data_state.selected();
                    }
                }
    
                if load_paginated {
                    app.dispatch_event(AppEvent::LoadIssueOpData);
                }
            }

            // If a ViewPanel is selected, scroll down on the View Panel
            else if let Some(view_panel_selected_idx) = app.linear_dashboard_view_panel_selected {
                // debug!("exec_scroll_down_cmd() view panel is selected");

                let view_panel_list_handle = app.linear_dashboard_view_panel_list.lock().unwrap();
                let view_panel_issue_handle = view_panel_list_handle[view_panel_selected_idx-1].issue_table_data.lock().unwrap();

                if let Some(table_state) = &app.view_panel_issue_selected {
                    // debug!("exec_scroll_down_cmd() view panel issue is selected");
                    let view_panel_cursor_handle = view_panel_list_handle[view_panel_selected_idx-1].view_cursor.lock().unwrap();

                    if !view_panel_issue_handle.is_empty() {
                        // Check if at end of app.view_panel_issue_selected
                        //  If true: Check if app.view_panel_list_handle[view_panel_selected_idx-1].view_loader.exhausted == false
                        //      If true: dispatch event to load next page view panel
                        //          and merge with current app.view_panel_list_handle[view_panel_selected_idx-1].issue_table_data

                        let is_last_element = table_state::is_last_element(table_state, &view_panel_issue_handle);
                        let cursor_is_exhausted = if let Some(cursor) = &*view_panel_cursor_handle {
                                cursor.platform == Platform::Linear && !cursor.has_next_page
                            }
                            else {
                                false
                            };

                        if is_last_element && !cursor_is_exhausted {

                            debug!("exec_scroll_down_cmd() at end of list with more to load, paginating");
                            app.view_panel_to_paginate = view_panel_selected_idx-1;
                            load_paginated = true;
                        }
                        // Not at end of list, scroll down
                        else {
                            // debug!("exec_scroll_down_cmd() attempting to scroll down");
                            app.view_panel_issue_selected = Some(table_state::with_next(table_state, &view_panel_issue_handle));
                        }
                    }
                }
                // If a View panel is selected && no issue is selected && panel has issues:
                //     select next issue
                else if !view_panel_issue_handle.is_empty() {
                    let mut table_state = TableState::default();
                    table_state::next(&mut table_state, &view_panel_issue_handle);

                    app.view_panel_issue_selected = Some( table_state );
                }
            }


            // No View Panel selected or issue modal open, scroll on actions
            else {
                app.actions.next();
            }

            // updated expanded issue
            if app.issue_to_expand.is_some() {
                exec_expand_issue_cmd(app);
            }

            if load_paginated {
                app.dispatch_event(AppEvent::PaginateDashboardView);
            }
        },

        // Select next Custom View Slot
        Route::DashboardViewDisplay => {
            if app.linear_dashboard_view_list_selected {
                table_state::next(&mut app.dashboard_view_display.view_table_state, &app.linear_dashboard_view_list);
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
                    let is_last_element = table_state::is_last_element(& app.linear_custom_view_select.view_table_state, handle);
                    let cursor_has_next_page;

                    {
                        let view_cursor_data_handle = app.linear_custom_view_cursor.lock().unwrap();
                        cursor_has_next_page = view_cursor_data_handle.has_next_page;
                    }

                    if is_last_element && cursor_has_next_page {
                        load_paginated = true;
                    }
                    else {
                        table_state::next(&mut app.linear_custom_view_select.view_table_state, handle);
                        app.linear_selected_custom_view_idx = app.linear_custom_view_select.view_table_state.selected();
                    }
                }
    
                if load_paginated {
                    app.dispatch_event(AppEvent::LoadCustomViews);
                }
            }
        }
    }
}

pub fn exec_scroll_up_cmd(app: &mut App) {

    let cur_route: Route = app.route.lock().unwrap().clone();

    match cur_route {
        Route::ConfigInterface => {
            // nothing to scroll
        },
        Route::ActionSelect => {

            // Don't scroll up if entering issue title
            if app.modifying_issue && app.linear_issue_op_interface.current_op == Some(IssueModificationOp::Title) { }

            // If the issue op interface is open, scroll down on modal
            else if app.modifying_issue {


                let obj_vec: Vec<IssueFieldObject> = match app.linear_issue_op_interface.table_data_from_op() {
                    Some(result) => result,
                    None => return,
                };
                let data_state = &mut app.linear_issue_op_interface.data_state;

                debug!("Attempting to scroll up on IssueOpInterface - data_lock, data_state: {:?}, {:?}", obj_vec, data_state);

                table_state::previous(data_state, &obj_vec);
                app.linear_issue_op_interface.selected_idx = app.linear_issue_op_interface.data_state.selected();
            }

            // If a ViewPanel is selected and no issue modal open, scroll up on the View Panel
            else if let Some(view_panel_selected_idx) = app.linear_dashboard_view_panel_selected {

                let view_panel_list_handle = app.linear_dashboard_view_panel_list.lock().unwrap();
                let view_panel_issue_handle = view_panel_list_handle[view_panel_selected_idx-1].issue_table_data.lock().unwrap();

                if let Some(table_state) = &app.view_panel_issue_selected {
                    if !view_panel_issue_handle.is_empty() {
                        app.view_panel_issue_selected = Some(table_state::with_previous(table_state, &view_panel_issue_handle));
                    }
                }
                // If a View panel is selected && no issue is selected && panel has issues:
                //     select next issue
                else if !view_panel_issue_handle.is_empty() {
                    let mut table_state = TableState::default();
                    table_state::next(&mut table_state, &view_panel_issue_handle);

                    app.view_panel_issue_selected = Some( table_state );
                }
            }
            // No View Panel selected or issue modal open, scroll on actions
            else {
                app.actions.next();
            }

            // updated expanded issue
            if app.issue_to_expand.is_some() {
                exec_expand_issue_cmd(app);
            }
        },
        // Select previous Custom View Slot
        Route::DashboardViewDisplay => {
            if app.linear_dashboard_view_list_selected {
                table_state::previous(&mut app.dashboard_view_display.view_table_state, &app.linear_dashboard_view_list);
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
                table_state::previous(&mut app.linear_custom_view_select.view_table_state, handle);
                app.linear_selected_custom_view_idx = app.linear_custom_view_select.view_table_state.selected();
            }
        }
    }
}