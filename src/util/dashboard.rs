
use crate::app::App;

use crate::linear::{ schema::{ Issue }, client::{ IssueFieldObject } };

// Accepts:
//     app
// Returns:
//     full JSON Issue object (as specified in GraphQL request), or None if a View Panel Issue is not selected
pub fn fetch_selected_view_panel_issue(app: &App) -> Option<Issue> {
    // Validate that a ViewPanel and issue are selected
    // using 'app.linear_dashboard_view_panel_selected' & 'app.view_panel_issue_selected'

    let selected_view_panel_idx: usize;
    let selected_issue_idx: usize;

    // Verify that a ViewPanel is selected
    if let Some(x) = app.linear_dashboard_view_panel_selected {
        selected_view_panel_idx = x;
    }
    else {
        return None;
    }

    // Verify that an issue is selected
    if let Some(issue_table_state) = &app.view_panel_issue_selected {
        if let Some(y) = issue_table_state.selected() {
            selected_issue_idx = y;
        }
        else {
            return None;
        }
    }
    else {
        return None;
    };

    // Fetch selected Issue
    let view_panel_list_handle = app.linear_dashboard_view_panel_list.lock().unwrap();

    let selected_view_panel_data_arc;
    if let Some(x) = view_panel_list_handle.get(selected_view_panel_idx-1) {
        selected_view_panel_data_arc = x.issue_table_data.clone();
        drop(view_panel_list_handle);
    }
    else {
        drop(view_panel_list_handle);
        return None;
    };

    let view_panel_data_handle = selected_view_panel_data_arc.lock().unwrap();

    let fetched_issue = view_panel_data_handle.get(selected_issue_idx).cloned();

    drop(view_panel_data_handle);

    fetched_issue

}

// Accepts:
//     app
// Returns:
//     Some(usize) is a ViewPanel is selected, None if a ViewPanel is not selected
//     Note: it is the index+1,
pub fn fetch_selected_view_panel_num(app: &App) -> Option<usize> {
    app.linear_dashboard_view_panel_selected
}


// Accepts:
//     app
// Returns:
//     Some(usize) is a ViewPanel is selected, None if a ViewPanel is not selected
//     Note: it is the zero-based index,
pub fn fetch_selected_view_panel_idx(app: &App) -> Option<usize> {
    app.linear_dashboard_view_panel_selected.map(|x| x-1)
}

// Accepts:
//     app
// Returns:
//     full JSON object (as specified in GraphQL request), or None if a Value is not selected
pub fn fetch_selected_value(app: &App) -> Option<IssueFieldObject> {

    let obj_vec = match app.linear_issue_op_interface.table_data_from_op() {
        Some(result) => result,
        _ => return None
    };
    let state_idx: usize = match app.linear_issue_op_interface.selected_idx {
        Some(x) => x,
        _ => return None
    };

    obj_vec.get(state_idx).cloned()
}