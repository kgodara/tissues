use crate::graphql::{
    parse_graphql_from_file,
};

use crate::errors::{
    GraphQLError,
    GraphQLRequestError
};

use serde_json::{
    Value,
    Map,
    Number
};

use serde_json::json;

use std::result::Result;

use crate::util::GraphQLCursor;
use crate::app::Platform;

const LINEAR_GET_VIEWER_PATH: &str = "queries/linear/get_viewer.graphql";
const LINEAR_FETCH_CUSTOM_VIEWS_PATH: &str = "queries/linear/fetch_custom_views.graphql";

const LINEAR_FETCH_ISSUES_BY_WORKFLOW_STATE_PATH: &str = "queries/linear/fetch_issues_by_workflow_state.graphql";
const LINEAR_FETCH_ISSUES_BY_ASSIGNEE_PATH: &str = "queries/linear/fetch_issues_by_assignee.graphql";
const LINEAR_FETCH_ISSUES_BY_LABEL_PATH: &str = "queries/linear/fetch_issues_by_label.graphql";
const LINEAR_FETCH_ISSUES_BY_CREATOR_PATH: &str = "queries/linear/fetch_issues_by_creator.graphql";


const LINEAR_GET_TEAMS_PATH: &str = "queries/linear/get_teams.graphql";
const LINEAR_FETCH_ISSUES_BY_TEAM_PATH: &str = "queries/linear/fetch_issues_by_team.graphql";
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

    match issue_cursor {
        Some(cursor_data) => {
            // If Cursor is for a different platform, and is not a new cursor
            if cursor_data.platform != Platform::Linear && cursor_data.platform != Platform::Na {
                return Err(GraphQLRequestError::GraphQLInvalidCursor(cursor_data));
            }
            if cursor_data.has_next_page == true && cursor_data.platform == Platform::Linear {
                query["variables"]["afterCursor"] = Value::String(cursor_data.end_cursor);
            }
        },
        None => {}
    };

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


// Custom View Resolver Queries

pub async fn fetch_issues_by_workflow_state(api_key: &str, issue_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, issue_page_size: u32) -> QueryResult {
    let mut query;

    query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_WORKFLOW_STATE_PATH)?;

    // query["variables"] = Value::Object(variables);


    query["variables"] = Value::Object(variables);
    // query["variables"] = json!({});
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    match issue_cursor {
        Some(cursor_data) => {
            // If Cursor is for a different platform, and is not a new cursor
            if cursor_data.platform != Platform::Linear && cursor_data.platform != Platform::Na {
                return Err(GraphQLRequestError::GraphQLInvalidCursor(cursor_data));
            }
            if cursor_data.has_next_page == true && cursor_data.platform == Platform::Linear {
                query["variables"]["afterCursor"] = Value::String(cursor_data.end_cursor);
            }
        },
        None => {}
    };

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

    match issue_cursor {
        Some(cursor_data) => {
            // If Cursor is for a different platform, and is not a new cursor
            if cursor_data.platform != Platform::Linear && cursor_data.platform != Platform::Na {
                return Err(GraphQLRequestError::GraphQLInvalidCursor(cursor_data));
            }
            if cursor_data.has_next_page == true && cursor_data.platform == Platform::Linear {
                query["variables"]["afterCursor"] = Value::String(cursor_data.end_cursor);
            }
        },
        None => {}
    };

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

    match issue_cursor {
        Some(cursor_data) => {
            // If Cursor is for a different platform, and is not a new cursor
            if cursor_data.platform != Platform::Linear && cursor_data.platform != Platform::Na {
                return Err(GraphQLRequestError::GraphQLInvalidCursor(cursor_data));
            }
            if cursor_data.has_next_page == true && cursor_data.platform == Platform::Linear {
                query["variables"]["afterCursor"] = Value::String(cursor_data.end_cursor);
            }
        },
        None => {}
    };

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

    match issue_cursor {
        Some(cursor_data) => {
            // If Cursor is for a different platform, and is not a new cursor
            if cursor_data.platform != Platform::Linear && cursor_data.platform != Platform::Na {
                return Err(GraphQLRequestError::GraphQLInvalidCursor(cursor_data));
            }
            if cursor_data.has_next_page == true && cursor_data.platform == Platform::Linear {
                query["variables"]["afterCursor"] = Value::String(cursor_data.end_cursor);
            }
        },
        None => {}
    };

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
pub async fn get_issues_by_team(api_key: &str, issue_cursor: Option<GraphQLCursor>, issue_page_size: u32, team: Map<String, Value>) -> QueryResult {
    let mut query;
    query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_TEAM_PATH)?;

    query["variables"] = Value::Object(team);
    query["variables"]["firstNum"] = Value::Number(Number::from(issue_page_size));

    match issue_cursor {
        Some(cursor_data) => {
            if cursor_data.platform != Platform::Linear {
                return Err(GraphQLRequestError::GraphQLInvalidCursor(cursor_data));
            }
            if cursor_data.has_next_page == true {
                query["variables"]["afterCursor"] = Value::String(cursor_data.end_cursor);
            }
        },
        None => {}
    };

    info!("get_issues_by_team variables: {:?}", query["variables"]);



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