// NOTE: Rate limiting can cause flakiness
// TODO: create & blast away custom views each run

#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;
use insta;

use anyhow::Result;

#[macro_use]
extern crate lazy_static;

use std::{
    io,
    fs, fs::File,
    sync::{ Arc, Mutex, Once }
};

use rust_cli::{
    app::{ Platform },
    linear::{
        LinearConfig,
        client::{ LinearClient },
        schema::{ CustomView, CustomViewResponseData }
    },
    util::{ GraphQLCursor, error_panic },
};


macro_rules! aw {
    ($e:expr) => {
        tokio_test::block_on($e)
    };
}

lazy_static! {
    pub static ref LINEAR_CLIENT: Arc<Mutex<Option<LinearClient>>> = Arc::new(Mutex::new(None));
    pub static ref CUSTOM_VIEWS: Arc<Mutex<Vec<CustomView>>> = Arc::new(Mutex::new(Vec::new()));
}

const SELECTED_PRIORITY_VIEW_ID: &str = "cf9db35e-5eb9-475a-8ae0-8a130821ead0";
const SELECTED_PROJECT_VIEW_ID: &str = "c0c7c852-5f4c-4a57-8a55-a306d86368f6";
const SELECTED_TEAM_VIEW_ID: &str = "5a8a4fa5-cdae-4a62-bcf2-bc69e14fdeb2";
const SELECTED_CREATOR_VIEW_ID: &str = "5895b38b-d98c-4898-815c-97f166de3316";
const SELECTED_ASSIGNEE_VIEW_ID: &str = "1477aacd-465c-49d3-9e14-a3b7952f4e22";

// Due Date Views
const OVER_DUE_VIEW_ID: &str = "52719a63-d7aa-4f1b-8157-91103ba51e0f";
const NO_DUE_DATE_VIEW_ID: &str = "3dfa04a4-ce78-45cd-882b-866774faee50";
const DUE_DATE_BEFORE_VIEW_ID: &str = "ee372cb9-6e3d-4da4-b7b7-003013293491";
const DUE_DATE_AFTER_VIEW_ID: &str = "2a19d661-73ca-4208-8fe0-5f3554892a60";


// Selected State Views
const SELECTED_STATE_VIEW_ID: &str = "aa09c686-9668-4104-87fc-58cdfea6fb8b";
const NOT_SELECTED_STATE_VIEW_ID: &str = "2fc599c6-19e4-44ba-bd58-2bcdc024fdea";
const SINGLE_SELECTED_STATE_VIEW_ID: &str = "8c5c35c0-8702-42ae-a74a-71afeb6a02f6";
const SINGLE_NOT_SELECTED_STATE_VIEW_ID: &str = "372f0ae9-035e-4314-97ee-f6614391df13";



static INIT: Once = Once::new();

#[cfg(test)]
fn setup_logger() {
    // setup logger
    let log_remove_result = fs::remove_file("rust_cli_test.log");

    match log_remove_result {
        Ok(_) => {},
        Err(x) => {
            match x.kind() {
                io::ErrorKind::NotFound => {},
                _ => panic!(),
            }
        }
    }

    WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("rust_cli_test.log").unwrap()).unwrap();
}

#[cfg(test)]
pub fn initialize() {
    INIT.call_once(|| {
        setup_logger();

        // Access Token found, continue
        match LinearConfig::load_config() {
            Some(config) => {
                // with_config() can return Err() if token file contains non visible ASCII chars (32-127)
                match LinearClient::with_config(config) {
                    Ok(mut client) => {
                        client.config.issue_page_size = 50;
                        client.config.custom_view_page_size = 50;

                        *LINEAR_CLIENT.lock().unwrap() = Some(client);

                    },
                    Err(_) => {
                        panic!("client init failed");
                    }
                }
            },
            None => {
                panic!("config load failed");
            }
        }
    
        // fetch all custom views
        let mut custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
        *custom_views_lock = fetch_all_custom_views();
    });
}


#[cfg(test)]
pub fn fetch_all_custom_views() -> Vec<CustomView> {

    let linear_client_lock = LINEAR_CLIENT.lock().unwrap();
    let client = linear_client_lock.as_ref().unwrap();

    // fetch all custom views
    let mut view_fetch_result: Result<Option<CustomViewResponseData>>;
    let mut custom_view_cursor: Option<GraphQLCursor> = None;

    let mut custom_views: Vec<CustomView> = Vec::new();

    loop {

        if let Some(ref cursor) = custom_view_cursor {
            if !cursor.has_next_page {
                break;
            }
        }

        view_fetch_result = aw!(client.custom_views(custom_view_cursor));


        if let Ok(Some(mut y)) = view_fetch_result {
            custom_views.append(&mut y.custom_views.nodes);

            custom_view_cursor = Some(GraphQLCursor {
                platform: Platform::Linear,
                has_next_page: y.custom_views.page_info.has_next_page,
                end_cursor: y
                    .custom_views
                    .page_info
                    .end_cursor,
            });
        } else {
            error_panic!("Error loading custom views: {:?}",view_fetch_result);
        }
    }

    debug!("fetch_all_custom_views() - fetched {} custom views", custom_views.len());
    custom_views
}

#[cfg(test)]
fn exec_snapshot_test(view: &CustomView, snapshot: &str)
{
    let linear_client_lock = LINEAR_CLIENT.lock().unwrap();
    let client = linear_client_lock.as_ref().unwrap();

    let mut view_issues_res = aw!(client.issues(serde_json::from_value(serde_json::to_value(view.filter_data.clone()).unwrap()).unwrap(), None));

    if let Ok(Some(view_issues)) = view_issues_res {
        debug!("exec_snapshot_test() - view_issues: {:?}", view_issues.issues.nodes);
        insta::assert_yaml_snapshot!(snapshot, view_issues.issues.nodes);
    } else {
        error_panic!("issues() query failed - {:?}", view_issues_res);
    }
}

// View Tests

#[test]
pub fn selected_priority_view() {
    initialize();
    
    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();

    let view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == SELECTED_PRIORITY_VIEW_ID
    }).expect("Failed to find SelectedPriority view");

    // SelectedPriority
    exec_snapshot_test(view, "selected_priority");
}

#[test]
pub fn selected_project_view() {
    initialize();

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();

    let view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == SELECTED_PROJECT_VIEW_ID
    }).expect("Failed to find SelectedProject view");

    // SelectedProject
    exec_snapshot_test(view, "selected_project");
}

#[test]
pub fn selected_team_view() {
    initialize();

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    
    let view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == SELECTED_TEAM_VIEW_ID
    }).expect("Failed to find SelectedTeam view");

    exec_snapshot_test(view, "selected_team");
}

#[test]
pub fn selected_creator_view() {
    initialize();

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();

    let view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == SELECTED_CREATOR_VIEW_ID
    }).expect("Failed to find SelectedCreator view");

    exec_snapshot_test(view, "selected_creator",);

}

#[test]
pub fn selected_assignee_view() {
    initialize();

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    
    let view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == SELECTED_ASSIGNEE_VIEW_ID
    }).expect("Failed to find SelectedAssignee view");

    exec_snapshot_test(view,"selected_assignee");
}

#[test]
pub fn due_date_views() {
    initialize();

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    
    let over_due_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == OVER_DUE_VIEW_ID
    }).expect("Failed to find OverDue view");

    let no_due_date_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == NO_DUE_DATE_VIEW_ID
    }).expect("Failed to find NoDueDate view");

    let due_date_before_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == DUE_DATE_BEFORE_VIEW_ID
    }).expect("Failed to find DueDateBefore view");

    let due_date_after_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == DUE_DATE_AFTER_VIEW_ID
    }).expect("Failed to find DueDateAfter view");

    // OverDue
    exec_snapshot_test(over_due_view, "due_date_over_due");

    // NoDueDate
    exec_snapshot_test(no_due_date_view, "due_date_no_due_date");

    // DueDateBefore
    exec_snapshot_test(due_date_before_view, "due_date_before");

    // DueDateAfter
    exec_snapshot_test(due_date_after_view, "due_date_after");

}

#[test]
pub fn workflow_state_views() {
    initialize();

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    

    let selected_state_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == SELECTED_STATE_VIEW_ID
    }).expect("Failed to find SelectedState view");

    let not_selected_state_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == NOT_SELECTED_STATE_VIEW_ID
    }).expect("Failed to find NotSelectedState view");

    let single_selected_state_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == SINGLE_SELECTED_STATE_VIEW_ID
    }).expect("Failed to find SingleSelectedState view");

    let single_not_selected_state_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == SINGLE_NOT_SELECTED_STATE_VIEW_ID
    }).expect("Failed to find SingleNotSelectedState view");


    // SelectedState
    exec_snapshot_test(
        selected_state_view,
        "workflow_state_selected",
    );

    // NotSelectedState
    exec_snapshot_test(
        not_selected_state_view,
        "workflow_state_not_selected",
    );

    // SingleSelectedState
    exec_snapshot_test(
        single_selected_state_view,
        "workflow_state_single_selected",
    );

    // SingleNotSelectedState
    exec_snapshot_test(
        single_not_selected_state_view,
        "workflow_state_single_not_selected",
    );

}