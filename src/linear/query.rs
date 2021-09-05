use crate::graphql::{
    parse_graphql_from_file,
};

use crate::errors::{
    GraphQLRequestError
};

use serde_json::{
    Value,
    Map,
    Number
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

const LINEAR_FETCH_CUSTOM_VIEWS_PATH: &str = "queries/linear/fetch_custom_views.graphql";
const LINEAR_FETCH_TEAM_TIME_ZONES_PATH: &str = "queries/linear/fetch_team_timezones.graphql";

const LINEAR_FETCH_ALL_ISSUES_PATH: &str = "queries/linear/issues/fetch_all_issues.graphql";
const LINEAR_FETCH_ISSUES_BY_TEAM_PATH: &str = "queries/linear/issues/fetch_issues_by_team.graphql";
const LINEAR_FETCH_ISSUES_BY_CONTENT_PATH: &str = "queries/linear/issues/fetch_issues_by_content.gql";
const LINEAR_FETCH_ISSUES_BY_WORKFLOW_STATE_PATH: &str = "queries/linear/issues/fetch_issues_by_workflow_state.graphql";
const LINEAR_FETCH_ISSUES_BY_ASSIGNEE_PATH: &str = "queries/linear/issues/fetch_issues_by_assignee.graphql";
const LINEAR_FETCH_ISSUES_BY_LABEL_PATH: &str = "queries/linear/issues/fetch_issues_by_label.graphql";
const LINEAR_FETCH_ISSUES_BY_CREATOR_PATH: &str = "queries/linear/issues/fetch_issues_by_creator.graphql";
const LINEAR_FETCH_ISSUES_BY_PROJECT_PATH: &str = "queries/linear/issues/fetch_issues_by_project.graphql";


const LINEAR_GET_WORKFLOW_STATES_BY_TEAM: &str = "queries/linear/op_fetch/get_workflow_states_by_team.graphql";
const LINEAR_GET_USERS_BY_TEAM: &str = "queries/linear/op_fetch/get_users_by_team.graphql";
const LINEAR_GET_PROJECTS_BY_TEAM: &str = "queries/linear/op_fetch/get_projects_by_team.graphql";
const LINEAR_GET_CYCLES_BY_TEAM: &str = "queries/linear/op_fetch/get_cycles_by_team.graphql";


const LINEAR_SET_ISSUE_WORKFLOW_STATE: &str = "queries/linear/issue_modifications/set_issue_workflow_state.graphql";
const LINEAR_SET_ISSUE_ASSIGNEE: &str = "queries/linear/issue_modifications/set_issue_assignee.graphql";
const LINEAR_SET_ISSUE_PROJECT: &str = "queries/linear/issue_modifications/set_issue_project.graphql";
const LINEAR_SET_ISSUE_CYCLE: &str = "queries/linear/issue_modifications/set_issue_cycle.graphql";

type QueryResult = Result<Value, GraphQLRequestError>;


pub async fn exec_fetch_custom_views(api_key: &str, issue_cursor: Option<GraphQLCursor>, issue_page_size: u32) -> QueryResult {
    let mut query;
    query = parse_graphql_from_file(&LINEAR_FETCH_CUSTOM_VIEWS_PATH)?;

    query["variables"] = Value::Object(Map::default());
    // query["variables"] = json!({});
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;

    info!("fetch_custom_views variables: {:?}", query["variables"]);

    let client = reqwest::Client::new();

    let resp = client.post("https://api.linear.app/graphql")
                        .header("Content-Type", "application/json")
                        .header("Authorization", api_key)
                        .json(&query)
                        .send()
                        .await?
                        .json()
                        .await?;

    Ok(resp)

}

pub async fn exec_fetch_team_timezones(api_key: &str, team_cursor: Option<GraphQLCursor>, team_tz_page_size: u32) -> QueryResult {
    let mut query;
    query = parse_graphql_from_file(&LINEAR_FETCH_TEAM_TIME_ZONES_PATH)?;

    query["variables"] = Value::Object(Map::default());
    // query["variables"] = json!({});
    query["variables"]["firstNum"] = Value::Number(Number::from(team_tz_page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], team_cursor)?;

    info!("fetch_team_timezones variables: {:?}", query["variables"]);

    let client = reqwest::Client::new();

    let resp = client.post("https://api.linear.app/graphql")
                        .header("Content-Type", "application/json")
                        .header("Authorization", api_key)
                        .json(&query)
                        .send()
                        .await?
                        .json()
                        .await?;

    Ok(resp)
}


// Custom View Resolver Queries

pub async fn exec_fetch_all_issues(api_key: &str, issue_cursor: Option<GraphQLCursor>, issue_page_size: u32) -> QueryResult {
    let mut query;

    query = parse_graphql_from_file(&LINEAR_FETCH_ALL_ISSUES_PATH)?;

    query["variables"] = Value::Object(Map::new());
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;

    info!("fetch_all_issues variables: {:?}", query["variables"]);

    let client = reqwest::Client::new();

    let resp = client.post("https://api.linear.app/graphql")
                        .header("Content-Type", "application/json")
                        .header("Authorization", api_key)
                        .json(&query)
                        .send()
                        .await?
                        .json()
                        .await?;
    Ok(resp)
}

pub async fn exec_fetch_issues_by_team(api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {
    let mut query;

    query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_TEAM_PATH)?;

    // query["variables"] = Value::Object(variables);


    query["variables"] = Value::Object(variables);
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));


    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;


    info!("fetch_issues_by_team variables: {:?}", query["variables"]);

    let client = reqwest::Client::new();

    let resp = client.post("https://api.linear.app/graphql")
                        .header("Content-Type", "application/json")
                        .header("Authorization", api_key)
                        .json(&query)
                        .send()
                        .await?
                        .json()
                        .await?;

    Ok(resp)
}


pub async fn exec_fetch_issue_by_direct_filter(filter_type: &FilterType, api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {
    let mut query;

    query = parse_graphql_from_file( match filter_type {
        FilterType::Content => &LINEAR_FETCH_ISSUES_BY_CONTENT_PATH,
        FilterType::SelectedState => &LINEAR_FETCH_ISSUES_BY_WORKFLOW_STATE_PATH,
        FilterType::SelectedCreator => &LINEAR_FETCH_ISSUES_BY_CREATOR_PATH,
        FilterType::SelectedAssignee => &LINEAR_FETCH_ISSUES_BY_ASSIGNEE_PATH,
        FilterType::SelectedLabel => &LINEAR_FETCH_ISSUES_BY_LABEL_PATH,
        FilterType::SelectedProject => &LINEAR_FETCH_ISSUES_BY_PROJECT_PATH,
        _ => {
            error!("exec_fetch_issue_by_direct_filter received unsupported FilterType: {:?}", filter_type);
            panic!("exec_fetch_issue_by_direct_filter received unsupported FilterType: {:?}", filter_type);
        }
    })?;

    query["variables"] = Value::Object(variables);
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    // Set "afterCursor" query variable
    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;


    info!("exec_fetch_issue_by_direct_filter - {:?} - variables: {:?}", filter_type, query["variables"]);

    let client = reqwest::Client::new();

    let resp = client.post("https://api.linear.app/graphql")
                        .header("Content-Type", "application/json")
                        .header("Authorization", api_key)
                        .json(&query)
                        .send()
                        .await?
                        .json()
                        .await?;

    Ok(resp)
}


// Non Custom View Resolver Queries

// Issue Op Queries

pub async fn exec_get_issue_op_data(op: &IssueModificationOp, api_key: &str, cursor: Option<GraphQLCursor>, variables: Map<String, Value>, page_size: u32) -> QueryResult {
    let mut query;

    query = parse_graphql_from_file(match op {
        IssueModificationOp::ModifyWorkflowState => &LINEAR_GET_WORKFLOW_STATES_BY_TEAM,
        IssueModificationOp::ModifyAssignee => &LINEAR_GET_USERS_BY_TEAM,
        IssueModificationOp::ModifyProject => &LINEAR_GET_PROJECTS_BY_TEAM,
        IssueModificationOp::ModifyCycle => &LINEAR_GET_CYCLES_BY_TEAM,
        _ => {
            error!("exec_get_issue_op_data - {:?} is unsupported", op);
            panic!("exec_get_issue_op_data - {:?} is unsupported", op);
        }
    })?;

    query["variables"] = Value::Object(variables);
    query["variables"]["firstNum"] = Value::Number(Number::from(page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], cursor)?;

    let client = reqwest::Client::new();

    let resp = client.post("https://api.linear.app/graphql")
                        .header("Content-Type", "application/json")
                        .header("Authorization", api_key)
                        .json(&query)
                        .send()
                        .await?
                        .json()
                        .await?;

    Ok(resp)
}

// Issue Op Update Mutations
pub async fn exec_update_issue(op: &IssueModificationOp, api_key: &str, variables: Map<String, Value>) -> QueryResult {
    let mut query;
    query = parse_graphql_from_file(match op {
        IssueModificationOp::ModifyWorkflowState => &LINEAR_SET_ISSUE_WORKFLOW_STATE,
        IssueModificationOp::ModifyAssignee => &LINEAR_SET_ISSUE_ASSIGNEE,
        IssueModificationOp::ModifyProject => &LINEAR_SET_ISSUE_PROJECT,
        IssueModificationOp::ModifyCycle => &LINEAR_SET_ISSUE_CYCLE,
        _ => {
            error!("exec_update_issue - {:?} is unsupported", op);
            panic!("exec_update_issue - {:?} is unsupported", op);
        }
    })?;

    query["variables"] = Value::Object(variables);

    info!("{:?} query: {:?}", op, query);

    let client = reqwest::Client::new();

    let resp = client.post("https://api.linear.app/graphql")
                        .header("Content-Type", "application/json")
                        .header("Authorization", api_key)
                        .json(&query)
                        .send()
                        .await?
                        .json()
                        .await?;

    Ok(resp)
}