
#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

#[macro_use]
extern crate lazy_static;

use std::{
    io,
    fs, fs::File,
    sync::{ Arc, Mutex, Once } 
};

use serde_json::{ Value };

use rust_cli::{ 
    // app::{ ALL_WORKFLOW_STATES, init_workflow_states },
    linear,
    linear::view_resolver_single_endpoint,
    util::{ GraphQLCursor },
};

use linear::{ LinearConfig, client::{ LinearClient, ClientResult } };

mod constants;
use constants::{ VIEW_ANSWER_KEY };

use tokio::runtime::Handle;


macro_rules! aw {
    ($e:expr) => {
        tokio_test::block_on($e)
    };
}

lazy_static! {
    pub static ref LINEAR_CLIENT: LinearClient = LinearClient::default();
    pub static ref CUSTOM_VIEWS: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
}

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
    
        // take a lock on config, and call load_config which has &mut self
        {
            let mut linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();
            if linear_config_lock.load_config().is_none() {
                panic!("Failed to load config");
            }
            // set issue_page_size to 50, to simplify pagination requirements
            linear_config_lock.issue_page_size = 50;
    
            // set custom_view_page_size to 50, to simplify pagination requirements
            linear_config_lock.custom_view_page_size = 50;
        }
    
        // fetch all custom views
        let mut custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
        *custom_views_lock = fetch_all_custom_views();
    });
}


#[cfg(test)]
pub fn fetch_all_custom_views() -> Vec<Value> {

    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();

    debug!("fetch_all_custom_views - linear_config_lock: {:?}", linear_config_lock);

    // fetch all custom views
    let mut view_fetch_result: ClientResult;
    let mut custom_view_cursor: Option<GraphQLCursor> = None;

    let mut custom_views: Vec<Value> = Vec::new();

    loop {

        let mut views: Value;
        let cursor_info: Value;

        if let Some(ref cursor) = custom_view_cursor {
            if !cursor.has_next_page {
                break;
            }
        }

        view_fetch_result = aw!(LinearClient::get_custom_views(linear_config_lock.clone(), custom_view_cursor));

        match view_fetch_result {
            Ok(x) => {
                views = x["view_nodes"].clone();
                cursor_info = x["cursor_info"].clone();
            },
            Err(y) => {
                error!("Get Custom Views failed: {:?}", y);
                panic!("Get Custom Views failed: {:?}", y);
            },
        }
        
        if let Some(new_views_vec) = views.as_array_mut() {
            custom_views.append(new_views_vec);
        }

        match GraphQLCursor::linear_cursor_from_page_info(cursor_info.clone()) {
            Some(z) => {
                // debug!("Updating view_cursor_data_lock to: {:?}", z);
                custom_view_cursor = Some(z);
            },
            None => {
                error!("'load_custom_views' linear_cursor_from_page_info() failed for cursor_info: {:?}", cursor_info);
                panic!("'load_custom_views' linear_cursor_from_page_info() failed for cursor_info: {:?}", cursor_info);
            },
        }
    }

    debug!("fetch_all_custom_views() - fetched {} custom views", custom_views.len());
    custom_views
}

#[cfg(test)]
pub fn validate_view_issues(view_issues: &[Value], view_id: &str) -> bool {
    for issue_id in VIEW_ANSWER_KEY[view_id].iter() {
            
        if !view_issues.iter().any(|issue| {
            &issue["id"].as_str().unwrap() == issue_id
        }) { return false; }
    }
    true
}

// View Tests
#[test]
pub fn selected_priority_view() {
    initialize();
    // cf9db35e-5eb9-475a-8ae0-8a130821ead0
    
    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();
    
    let view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "cf9db35e-5eb9-475a-8ae0-8a130821ead0"
    }).expect("Failed to find SelectedPriority view");

    let view_id = view["id"].as_str().expect("SelectedPriority view id not found");

    let view_issues: Vec<Value> = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&view.clone(), None, linear_config_lock.clone())).0;

    assert!(validate_view_issues(&view_issues, view_id));
}

#[test]
pub fn selected_project_view() {
    initialize();
    // c0c7c852-5f4c-4a57-8a55-a306d86368f6
    
    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();
    
    let view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "c0c7c852-5f4c-4a57-8a55-a306d86368f6"
    }).expect("Failed to find SelectedProject view");

    let view_id = view["id"].as_str().expect("SelectedProject view id not found");

    let view_issues: Vec<Value> = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&view.clone(), None, linear_config_lock.clone())).0;

    assert!(validate_view_issues(&view_issues, view_id));
}

#[test]
pub fn selected_team_view() {
    initialize();
    // 5a8a4fa5-cdae-4a62-bcf2-bc69e14fdeb2
    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();
    
    let view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "5a8a4fa5-cdae-4a62-bcf2-bc69e14fdeb2"
    }).expect("Failed to find SelectedTeam view");

    let view_id = view["id"].as_str().expect("SelectedTeam view id not found");

    let view_issues: Vec<Value> = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&view.clone(), None, linear_config_lock.clone())).0;

    assert!(validate_view_issues(&view_issues, view_id));
}

#[test]
pub fn selected_creator_view() {
    initialize();
    // 5895b38b-d98c-4898-815c-97f166de3316
    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();
    
    let view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "5895b38b-d98c-4898-815c-97f166de3316"
    }).expect("Failed to find SelectedCreator view");

    let view_id = view["id"].as_str().expect("SelectedCreator view id not found");

    let view_issues: Vec<Value> = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&view.clone(), None, linear_config_lock.clone())).0;

    assert!(validate_view_issues(&view_issues, view_id));
}

#[test]
pub fn selected_assignee_view() {
    initialize();
    // 1477aacd-465c-49d3-9e14-a3b7952f4e22
    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();

    let view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "1477aacd-465c-49d3-9e14-a3b7952f4e22"
    }).expect("Failed to find SelectedAssignee view");

    let view_id = view["id"].as_str().expect("SelectedAssignee view id not found");

    let view_issues: Vec<Value> = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&view.clone(), None, linear_config_lock.clone())).0;

    assert!(validate_view_issues(&view_issues, view_id));
}

#[test]
pub fn due_date_views() {
    initialize();

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();

    let over_due_view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "52719a63-d7aa-4f1b-8157-91103ba51e0f"
    }).expect("Failed to find OverDue view");

    let no_due_date_view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "3dfa04a4-ce78-45cd-882b-866774faee50"
    }).expect("Failed to find NoDueDate view");

    let due_date_before_view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "ee372cb9-6e3d-4da4-b7b7-003013293491"
    }).expect("Failed to find DueDateBefore view");

    let due_date_after_view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "2a19d661-73ca-4208-8fe0-5f3554892a60"
    }).expect("Failed to find DueDateAfter view");

    // OverDue
    let mut view_id = over_due_view["id"].as_str().expect("OverDue view id not found");
    let mut view_issues: Vec<Value> = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&over_due_view.clone(), None, linear_config_lock.clone())).0;
    debug!("OverDue view issues: {:?}", view_issues);
    assert!(validate_view_issues(&view_issues, view_id));

    // NoDueDate
    view_id = no_due_date_view["id"].as_str().expect("NoDueDate view id not found");
    view_issues = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&no_due_date_view.clone(), None, linear_config_lock.clone())).0;
    debug!("NoDueDate view issues: {:?}", view_issues);
    assert!(validate_view_issues(&view_issues, view_id));

    // DueDateBefore
    view_id = due_date_before_view["id"].as_str().expect("DueDateBefore view id not found");
    view_issues = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&due_date_before_view.clone(), None, linear_config_lock.clone())).0;
    debug!("DueDateBefore view issues: {:?}", view_issues);
    assert!(validate_view_issues(&view_issues, view_id));

    // DueDateAfter
    view_id = due_date_after_view["id"].as_str().expect("DueDateAfter view id not found");
    view_issues = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&due_date_after_view.clone(), None, linear_config_lock.clone())).0;
    debug!("DueDateAfter view issues: {:?}", view_issues);
    assert!(validate_view_issues(&view_issues, view_id));
}


#[test]
pub fn workflow_state_views() {
    initialize();

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();

    let selected_state_view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "aa09c686-9668-4104-87fc-58cdfea6fb8b"
    }).expect("Failed to find SelectedState view");

    let not_selected_state_view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "2fc599c6-19e4-44ba-bd58-2bcdc024fdea"
    }).expect("Failed to find NotSelectedState view");

    let single_selected_state_view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "8c5c35c0-8702-42ae-a74a-71afeb6a02f6"
    }).expect("Failed to find SingleSelectedState view");

    let single_not_selected_state_view: &Value = custom_views_lock.iter().find(|view| {
        view["id"].as_str().unwrap() == "372f0ae9-035e-4314-97ee-f6614391df13"
    }).expect("Failed to find SingleNotSelectedState view");

    // SelectedState
    let mut view_id = selected_state_view["id"].as_str().expect("SelectedState view id not found");
    let mut view_issues: Vec<Value> = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&selected_state_view.clone(), None, linear_config_lock.clone())).0;
    debug!("SelectedState view issues: {:?}", view_issues);
    assert!(validate_view_issues(&view_issues, view_id));

    
    // NotSelectedState
    view_id = not_selected_state_view["id"].as_str().expect("NotSelectedState view id not found");
    view_issues = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&not_selected_state_view.clone(), None, linear_config_lock.clone())).0;
    debug!("NotSelectedState view issues: {:?}", view_issues);
    assert!(validate_view_issues(&view_issues, view_id));

    // SingleSelectedState
    view_id = single_selected_state_view["id"].as_str().expect("SingleSelectedState view id not found");
    view_issues = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&single_selected_state_view.clone(), None, linear_config_lock.clone())).0;
    debug!("SingleSelectedState view issues: {:?}", view_issues);
    assert!(validate_view_issues(&view_issues, view_id));


    // SingleNotSelectedState
    view_id = single_not_selected_state_view["id"].as_str().expect("SingleNotSelectedState view id not found");
    view_issues = aw!(view_resolver_single_endpoint::optimized_view_issue_fetch(&single_not_selected_state_view.clone(), None, linear_config_lock.clone())).0;
    debug!("SingleNotSelectedState view issues: {:?}", view_issues);
    assert!(validate_view_issues(&view_issues, view_id));
}