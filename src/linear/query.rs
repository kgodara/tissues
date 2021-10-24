use crate::errors::{
    GraphQLRequestError
};

use serde_json::{
    Value,
    Map,
    Number,
    from_str,
};

use std::result::Result;

use crate::util::{
    GraphQLCursor,
    set_linear_after_cursor_from_opt
};

use crate::linear::view_resolver::FilterType;

use crate::constants::{
    IssueModificationOp
};

include!(concat!(env!("OUT_DIR"), "/query_raw.rs"));

lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::new();

    pub static ref LINEAR_FETCH_CUSTOM_VIEWS: Value = from_str(FETCH_CUSTOM_VIEWS).unwrap();
    pub static ref LINEAR_FETCH_TEAM_TIME_ZONES: Value = from_str(FETCH_TEAM_TIMEZONES).unwrap();
    pub static ref LINEAR_FETCH_VIEWER: Value = from_str(FETCH_VIEWER).unwrap();

    pub static ref LINEAR_FETCH_ALL_ISSUES: Value = from_str(FETCH_ALL_ISSUES).unwrap();
    pub static ref LINEAR_FETCH_ISSUES_BY_TEAM: Value = from_str(FETCH_ISSUES_BY_TEAM).unwrap();
    pub static ref LINEAR_FETCH_ISSUES_BY_CONTENT: Value = from_str(FETCH_ISSUES_BY_CONTENT).unwrap();
    pub static ref LINEAR_FETCH_ISSUES_BY_WORKFLOW_STATE: Value = from_str(FETCH_ISSUES_BY_WORKFLOW_STATE).unwrap();
    pub static ref LINEAR_FETCH_ISSUES_BY_ASSIGNEE: Value = from_str(FETCH_ISSUES_BY_ASSIGNEE).unwrap();
    pub static ref LINEAR_FETCH_ISSUES_BY_LABEL: Value = from_str(FETCH_ISSUES_BY_LABEL).unwrap();
    pub static ref LINEAR_FETCH_ISSUES_BY_CREATOR: Value = from_str(FETCH_ISSUES_BY_CREATOR).unwrap();
    pub static ref LINEAR_FETCH_ISSUES_BY_PROJECT: Value = from_str(FETCH_ISSUES_BY_PROJECT).unwrap();


    pub static ref LINEAR_GET_WORKFLOW_STATES_BY_TEAM: Value = from_str(GET_WORKFLOW_STATES_BY_TEAM).unwrap();
    pub static ref LINEAR_GET_USERS_BY_TEAM: Value = from_str(GET_USERS_BY_TEAM).unwrap();
    pub static ref LINEAR_GET_PROJECTS_BY_TEAM: Value = from_str(GET_PROJECTS_BY_TEAM).unwrap();
    pub static ref LINEAR_GET_CYCLES_BY_TEAM: Value = from_str(GET_CYCLES_BY_TEAM).unwrap();


    pub static ref LINEAR_SET_ISSUE_WORKFLOW_STATE: Value = from_str(SET_ISSUE_WORKFLOW_STATE).unwrap();
    pub static ref LINEAR_SET_ISSUE_ASSIGNEE: Value = from_str(SET_ISSUE_ASSIGNEE).unwrap();
    pub static ref LINEAR_SET_ISSUE_PROJECT: Value = from_str(SET_ISSUE_PROJECT).unwrap();
    pub static ref LINEAR_SET_ISSUE_CYCLE: Value = from_str(SET_ISSUE_CYCLE).unwrap();
    pub static ref LINEAR_SET_ISSUE_TITLE: Value = from_str(SET_ISSUE_TITLE).unwrap();
}


type QueryResult = Result<Value, GraphQLRequestError>;

async fn dispatch_linear_req(api_key: &str, query: &Value) -> QueryResult {
    let r = CLIENT.post("https://api.linear.app/graphql")
                        .header("Content-Type", "application/json")
                        .header("Authorization", api_key)
                        .json(&query)
                        .send()
                        .await?
                        .json()
                        .await?;

    Ok(r)
}


pub async fn exec_fetch_custom_views(api_key: &str, issue_cursor: Option<GraphQLCursor>, issue_page_size: u32) -> QueryResult {

    let mut query = LINEAR_FETCH_CUSTOM_VIEWS.clone();

    query["variables"] = Value::Object(Map::default());
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;

    info!("fetch_custom_views variables: {:?}", query["variables"]);

    dispatch_linear_req(api_key, &query).await
}

pub async fn exec_fetch_team_timezones(api_key: &str, team_cursor: Option<GraphQLCursor>, team_tz_page_size: u32) -> QueryResult {
    let mut query = LINEAR_FETCH_TEAM_TIME_ZONES.clone();

    query["variables"] = Value::Object(Map::default());
    query["variables"]["firstNum"] = Value::Number(Number::from(team_tz_page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], team_cursor)?;

    info!("fetch_team_timezones variables: {:?}", query["variables"]);

    dispatch_linear_req(api_key, &query).await
}

pub async fn exec_fetch_viewer(api_key: &str) -> QueryResult {
    let mut query = LINEAR_FETCH_VIEWER.clone();

    dispatch_linear_req(api_key, &query).await
}


// Custom View Resolver Queries

pub async fn exec_fetch_all_issues(api_key: &str, issue_cursor: Option<GraphQLCursor>, issue_page_size: u32) -> QueryResult {
    let mut query = LINEAR_FETCH_ALL_ISSUES.clone();

    // query = parse_graphql_from_file(&LINEAR_FETCH_ALL_ISSUES_PATH)?;

    query["variables"] = Value::Object(Map::new());
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;

    info!("fetch_all_issues variables: {:?}", query["variables"]);

    dispatch_linear_req(api_key, &query).await
}

pub async fn exec_fetch_issues_by_team(api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {
    let mut query = LINEAR_FETCH_ISSUES_BY_TEAM.clone();

    // query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_TEAM_PATH)?;

    query["variables"] = Value::Object(variables);
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));


    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;


    info!("fetch_issues_by_team variables: {:?}", query["variables"]);

    dispatch_linear_req(api_key, &query).await
}


pub async fn exec_fetch_issue_by_direct_filter(filter_type: &FilterType, api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {

    let mut query: Value = match filter_type {
        FilterType::Content => LINEAR_FETCH_ISSUES_BY_CONTENT.clone(),
        FilterType::SelectedState => LINEAR_FETCH_ISSUES_BY_WORKFLOW_STATE.clone(),
        FilterType::SelectedCreator => LINEAR_FETCH_ISSUES_BY_CREATOR.clone(),
        FilterType::SelectedAssignee => LINEAR_FETCH_ISSUES_BY_ASSIGNEE.clone(),
        FilterType::SelectedLabel => LINEAR_FETCH_ISSUES_BY_LABEL.clone(),
        FilterType::SelectedProject => LINEAR_FETCH_ISSUES_BY_PROJECT.clone(),
        _ => {
            error!("exec_fetch_issue_by_direct_filter received unsupported FilterType: {:?}", filter_type);
            panic!("exec_fetch_issue_by_direct_filter received unsupported FilterType: {:?}", filter_type);
        }
    };

    query["variables"] = Value::Object(variables);
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    // Set "afterCursor" query variable
    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;


    info!("exec_fetch_issue_by_direct_filter - {:?} - variables: {:?}", filter_type, query["variables"]);

    dispatch_linear_req(api_key, &query).await
}


// Non Custom View Resolver Queries

// Issue Op Queries

pub async fn exec_get_issue_op_data(op: &IssueModificationOp, api_key: &str, cursor: Option<GraphQLCursor>, variables: Map<String, Value>, page_size: u32) -> QueryResult {

    let mut query: Value = match op {
        IssueModificationOp::WorkflowState => LINEAR_GET_WORKFLOW_STATES_BY_TEAM.clone(),
        IssueModificationOp::Assignee => LINEAR_GET_USERS_BY_TEAM.clone(),
        IssueModificationOp::Project => LINEAR_GET_PROJECTS_BY_TEAM.clone(),
        IssueModificationOp::Cycle => LINEAR_GET_CYCLES_BY_TEAM.clone(),
        _ => {
            error!("exec_get_issue_op_data - {:?} is unsupported", op);
            panic!("exec_get_issue_op_data - {:?} is unsupported", op);
        }
    };

    query["variables"] = Value::Object(variables);
    query["variables"]["firstNum"] = Value::Number(Number::from(page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], cursor)?;

    dispatch_linear_req(api_key, &query).await
}

// Issue Op Update Mutations
pub async fn exec_update_issue(op: &IssueModificationOp, api_key: &str, variables: Map<String, Value>) -> QueryResult {

    let mut query: Value = match op {
        IssueModificationOp::Title => LINEAR_SET_ISSUE_TITLE.clone(),
        IssueModificationOp::WorkflowState => LINEAR_SET_ISSUE_WORKFLOW_STATE.clone(),
        IssueModificationOp::Assignee => LINEAR_SET_ISSUE_ASSIGNEE.clone(),
        IssueModificationOp::Project => LINEAR_SET_ISSUE_PROJECT.clone(),
        IssueModificationOp::Cycle => LINEAR_SET_ISSUE_CYCLE.clone(),
        _ => {
            error!("exec_update_issue - {:?} is unsupported", op);
            panic!("exec_update_issue - {:?} is unsupported", op);
        }
    };

    query["variables"] = Value::Object(variables);

    info!("{:?} query: {:?}", op, query);

    dispatch_linear_req(api_key, &query).await
}