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

use crate::util::GraphQLCursor;

use crate::util::set_linear_after_cursor_from_opt;

const LINEAR_GET_VIEWER_PATH: &str = "queries/linear/get_viewer.graphql";
const LINEAR_FETCH_CUSTOM_VIEWS_PATH: &str = "queries/linear/fetch_custom_views.graphql";
const LINEAR_FETCH_TEAM_TIME_ZONES_PATH: &str = "queries/linear/fetch_team_timezones.graphql";

const LINEAR_FETCH_ALL_ISSUES_PATH: &str = "queries/linear/issues/fetch_all_issues.graphql";
const LINEAR_FETCH_ISSUES_BY_TEAM_PATH: &str = "queries/linear/issues/fetch_issues_by_team.graphql";
const LINEAR_FETCH_ISSUES_BY_WORKFLOW_STATE_PATH: &str = "queries/linear/issues/fetch_issues_by_workflow_state.graphql";
const LINEAR_FETCH_ISSUES_BY_ASSIGNEE_PATH: &str = "queries/linear/issues/fetch_issues_by_assignee.graphql";
const LINEAR_FETCH_ISSUES_BY_LABEL_PATH: &str = "queries/linear/issues/fetch_issues_by_label.graphql";
const LINEAR_FETCH_ISSUES_BY_CREATOR_PATH: &str = "queries/linear/issues/fetch_issues_by_creator.graphql";
const LINEAR_FETCH_ISSUES_BY_PROJECT: &str = "queries/linear/issues/fetch_issues_by_project.graphql";


const LINEAR_GET_TEAMS_PATH: &str = "queries/linear/get_teams.graphql";
const LINEAR_FETCH_ISSUES_BY_TEAM_PATH_OLD: &str = "queries/linear/fetch_issues_by_team.graphql";
const LINEAR_GET_WORKFLOW_STATES_BY_TEAM: &str = "queries/linear/get_workflow_states_by_team.graphql";
const LINEAR_UPDATE_ISSUE_WORKFLOW_STATE: &str = "queries/linear/update_issue_workflow_state.graphql";

type QueryResult = Result<Value, GraphQLRequestError>;

pub async fn get_viewer(api_key: &str) -> QueryResult {


    let query;
    query = parse_graphql_from_file(&LINEAR_GET_VIEWER_PATH)?;


    /*
    // Requires the `json` feature enabled.
    let resp: Value = ureq::post("https://api.linear.app/graphql")
                            .set("Content-Type", "application/json")
                            .set("Authorization", api_key)
                            .send_json(query)?
                            .into_json()?;
                            //.into_string()?;
    */

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

pub async fn fetch_custom_views(api_key: &str, issue_cursor: Option<GraphQLCursor>, issue_page_size: u32) -> QueryResult {
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

pub async fn fetch_team_timezones(api_key: &str, team_cursor: Option<GraphQLCursor>, team_tz_page_size: u32) -> QueryResult {
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

pub async fn fetch_all_issues(api_key: &str, issue_cursor: Option<GraphQLCursor>, issue_page_size: u32) -> QueryResult {
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

pub async fn fetch_issues_by_team(api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {
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

pub async fn fetch_issues_by_workflow_state(api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {
    let mut query;

    query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_WORKFLOW_STATE_PATH)?;

    // query["variables"] = Value::Object(variables);


    query["variables"] = Value::Object(variables);
    // query["variables"] = json!({});
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;

    info!("fetch_issues_by_workflow_state variables: {:?}", query["variables"]);

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

pub async fn fetch_issues_by_assignee(api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {
    let mut query;

    query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_ASSIGNEE_PATH)?;

    query["variables"] = Value::Object(variables);
    // query["variables"] = json!({});
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;

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

pub async fn fetch_issues_by_label(api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {
    let mut query;

    query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_LABEL_PATH)?;

    query["variables"] = Value::Object(variables);
    // query["variables"] = json!({});
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;

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

pub async fn fetch_issues_by_creator(api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {
    let mut query;

    query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_CREATOR_PATH)?;

    query["variables"] = Value::Object(variables);
    // query["variables"] = json!({});
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;

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

pub async fn fetch_issues_by_project(api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {
    let mut query;

    query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_PROJECT)?;

    query["variables"] = Value::Object(variables);
    // query["variables"] = json!({});
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));


    set_linear_after_cursor_from_opt(&mut query["variables"], issue_cursor)?;

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


pub async fn get_teams(api_key: &str) -> QueryResult {


    let query;
    query = parse_graphql_from_file(&LINEAR_GET_TEAMS_PATH)?;


    /*
    // Requires the `json` feature enabled.
    let resp: Value = ureq::post("https://api.linear.app/graphql")
                            .set("Content-Type", "application/json")
                            .set("Authorization", api_key)
                            .send_json(query)?
                            .into_json()?;
                            /*.into_string()?;*/
    */

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

pub async fn get_workflow_states_by_team(api_key: &str, variables: Map<String, Value>) -> QueryResult {

    let mut query;

    query = parse_graphql_from_file(&LINEAR_GET_WORKFLOW_STATES_BY_TEAM)?;

    query["variables"] = serde_json::Value::Object(variables);

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

pub async fn update_issue_workflow_state(api_key: &str, variables: Map<String, Value>) -> QueryResult {

    let mut query;
    query = parse_graphql_from_file(&LINEAR_UPDATE_ISSUE_WORKFLOW_STATE)?;

    query["variables"] = serde_json::Value::Object(variables);

    info!("update_issue_workflow_state query: {:?}", query);

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