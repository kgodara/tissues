
#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;
use insta;
use serde_json::json;

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
    linear::view_resolver,
    util::{ GraphQLCursor },
};

use linear::{ LinearConfig, client::{ LinearClient, ClientResult }, types::{ CustomView } };


macro_rules! aw {
    ($e:expr) => {
        tokio_test::block_on($e)
    };
}

lazy_static! {
    pub static ref LINEAR_CLIENT: LinearClient = LinearClient::default();
    pub static ref CUSTOM_VIEWS: Arc<Mutex<Vec<CustomView>>> = Arc::new(Mutex::new(Vec::new()));
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
        *custom_views_lock = serde_json::from_value(Value::Array(fetch_all_custom_views())).unwrap();
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
fn exec_snapshot_test<F>(view: &CustomView, linear_config: LinearConfig, snapshot: &str, issue_field_remapper: Option<F>)
where F: Fn(Value) -> Value
{
    let mut view_issues: Vec<Value> = aw!(view_resolver::optimized_view_issue_fetch(&view.clone(), None, linear_config)).0;
    if let Some(field_remapper) = issue_field_remapper {
        view_issues = view_issues.into_iter().map(field_remapper).collect();
    }
    insta::assert_yaml_snapshot!(snapshot, view_issues);
}

// View Tests

#[test]
pub fn selected_priority_view() {
    initialize();
    // cf9db35e-5eb9-475a-8ae0-8a130821ead0

    /* only care about:
        id
        createdAt
        number
        priority
    */

    let strip_priority = |issue_value: Value| {
        json!({
            "id": issue_value["id"],
            "createdAt": issue_value["createdAt"],
            "number": issue_value["number"],
            "priority": issue_value["priority"],
        })
    };
    
    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();

    let view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "cf9db35e-5eb9-475a-8ae0-8a130821ead0"
    }).expect("Failed to find SelectedPriority view");

    // SelectedPriority
    exec_snapshot_test(
        view,
        linear_config_lock.clone(),
        "selected_priority",
        Some(strip_priority)
    );
}

#[test]
pub fn selected_project_view() {
    initialize();
    // c0c7c852-5f4c-4a57-8a55-a306d86368f6

    /* only care about:
        id
        createdAt
        number
        project.id
    */

    let strip_project = |issue_value: Value| {
        json!({
            "id": issue_value["id"],
            "createdAt": issue_value["createdAt"],
            "number": issue_value["number"],
            "project": { "id": issue_value["project"]["id"] },
        })
    };

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();

    let view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "c0c7c852-5f4c-4a57-8a55-a306d86368f6"
    }).expect("Failed to find SelectedProject view");

    // SelectedProject
    exec_snapshot_test(
        view,
        linear_config_lock.clone(),
        "selected_project",
        Some(strip_project)
    );
}

#[test]
pub fn selected_team_view() {
    initialize();

    /* only care about:
        id
        createdAt
        number
        team.id
    */

    let strip_team = |issue_value: Value| {
        json!({
            "id": issue_value["id"],
            "createdAt": issue_value["createdAt"],
            "number": issue_value["number"],
            "team": { "id":  issue_value["team"]["id"]},
        })
    };

    // 5a8a4fa5-cdae-4a62-bcf2-bc69e14fdeb2
    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();
    
    let view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "5a8a4fa5-cdae-4a62-bcf2-bc69e14fdeb2"
    }).expect("Failed to find SelectedTeam view");

    exec_snapshot_test(view,
        linear_config_lock.clone(),
        "selected_team",
        Some(strip_team));
}

#[test]
pub fn selected_creator_view() {
    initialize();

    /* only care about (creator.id not in default query):
        id
        createdAt
        number
    */

    let strip_creator = |issue_value: Value| {
        json!({
            "id": issue_value["id"],
            "createdAt": issue_value["createdAt"],
            "number": issue_value["number"],
        })
    };

    // 5895b38b-d98c-4898-815c-97f166de3316
    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();

    let view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "5895b38b-d98c-4898-815c-97f166de3316"
    }).expect("Failed to find SelectedCreator view");

    exec_snapshot_test(
        view,
        linear_config_lock.clone(),
        "selected_creator",
        Some(strip_creator));

}

#[test]
pub fn selected_assignee_view() {
    initialize();

    /* only care about:
        id
        createdAt
        number
        assignee.id
    */

    let strip_assignee = |issue_value: Value| {
        json!({
            "id": issue_value["id"],
            "createdAt": issue_value["createdAt"],
            "number": issue_value["number"],
            "assignee": { "id":  issue_value["assignee"]["id"]},
        })
    };

    // 1477aacd-465c-49d3-9e14-a3b7952f4e22
    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();

    let view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "1477aacd-465c-49d3-9e14-a3b7952f4e22"
    }).expect("Failed to find SelectedAssignee view");

    exec_snapshot_test(
        view,
        linear_config_lock.clone(),
        "selected_assignee",
        Some(strip_assignee)
    );

}

#[test]
pub fn due_date_views() {
    initialize();

    /* only care about:
        id
        createdAt
        number
        dueDate
    */

    let strip_due_date = |issue_value: Value| {
        json!({
            "id": issue_value["id"],
            "createdAt": issue_value["createdAt"],
            "number": issue_value["number"],
            "dueDate": issue_value["dueDate"],
        })
    };

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();

    let over_due_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "52719a63-d7aa-4f1b-8157-91103ba51e0f"
    }).expect("Failed to find OverDue view");

    let no_due_date_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "3dfa04a4-ce78-45cd-882b-866774faee50"
    }).expect("Failed to find NoDueDate view");

    let due_date_before_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "ee372cb9-6e3d-4da4-b7b7-003013293491"
    }).expect("Failed to find DueDateBefore view");

    let due_date_after_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "2a19d661-73ca-4208-8fe0-5f3554892a60"
    }).expect("Failed to find DueDateAfter view");

    // OverDue
    exec_snapshot_test(
        over_due_view,
        linear_config_lock.clone(),
        "due_date_over_due",
        Some(strip_due_date)
    );

    // NoDueDate
    exec_snapshot_test(
        no_due_date_view,
        linear_config_lock.clone(),
        "due_date_no_due_date",
        Some(strip_due_date)
    );

    // DueDateBefore
    exec_snapshot_test(
        due_date_before_view,
        linear_config_lock.clone(),
        "due_date_before",
        Some(strip_due_date)
    );

    // DueDateAfter
    exec_snapshot_test(
        due_date_after_view,
        linear_config_lock.clone(),
        "due_date_after",
        Some(strip_due_date)
    );

}

#[test]
pub fn workflow_state_views() {
    initialize();

    /* only care about:
        id
        createdAt
        number
        state.id
    */
    /* Equivalent redactions object expression:
    {
        "[].dueDate" => "[dueDate]",
        ...
    }
    */

    let strip_state = |issue_value: Value| {
        json!({
            "id": issue_value["id"],
            "createdAt": issue_value["createdAt"],
            "number": issue_value["number"],
            "state": { "id":  issue_value["state"]["id"]},
        })
    };

    let custom_views_lock = CUSTOM_VIEWS.lock().unwrap();
    let linear_config_lock = LINEAR_CLIENT.config.lock().unwrap();

    let selected_state_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "aa09c686-9668-4104-87fc-58cdfea6fb8b"
    }).expect("Failed to find SelectedState view");

    let not_selected_state_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "2fc599c6-19e4-44ba-bd58-2bcdc024fdea"
    }).expect("Failed to find NotSelectedState view");

    let single_selected_state_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "8c5c35c0-8702-42ae-a74a-71afeb6a02f6"
    }).expect("Failed to find SingleSelectedState view");

    let single_not_selected_state_view: &CustomView = custom_views_lock.iter().find(|view| {
        view.id == "372f0ae9-035e-4314-97ee-f6614391df13"
    }).expect("Failed to find SingleNotSelectedState view");


    // SelectedState
    exec_snapshot_test(
        selected_state_view,
        linear_config_lock.clone(), 
        "workflow_state_selected",
        Some(strip_state)
    );

    // NotSelectedState
    exec_snapshot_test(
        not_selected_state_view,
        linear_config_lock.clone(), 
        "workflow_state_not_selected",
        Some(strip_state)
    );

    // SingleSelectedState
    exec_snapshot_test(
        single_selected_state_view,
        linear_config_lock.clone(), 
        "workflow_state_single_selected",
        Some(strip_state)
    );

    // SingleNotSelectedState
    exec_snapshot_test(
        single_not_selected_state_view,
        linear_config_lock.clone(), 
        "workflow_state_single_not_selected",
        Some(strip_state)
    );

}