
use serde_json::Value;

use crate::app::App;

pub fn fetch_selected_view_panel_issue(app: &App) -> Option<Value> {
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

pub fn fetch_selected_workflow_state(app: &App) -> Option<Value> {
    let workflow_state_data_handle = app.linear_workflow_select.workflow_states_data.lock().unwrap();
    let state_idx: usize;

    if let Some(x) = app.linear_selected_workflow_state_idx {
        state_idx = x;
    }
    else {
        return None;
    }

    workflow_state_data_handle.get(state_idx).cloned()
}