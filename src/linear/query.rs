use crate::graphql::{
    parse_graphql_from_file,
};

use crate::errors::{
    GraphQLError,
    GraphQLRequestError
};

use serde_json::{
    Value
};

use std::result::Result;

const LINEAR_GET_VIEWER_PATH: &str = "queries/linear/get_viewer.graphql";
const LINEAR_GET_TEAMS_PATH: &str = "queries/linear/get_teams.graphql";
const LINEAR_FETCH_ISSUES_BY_TEAM_PATH: &str = "queries/linear/fetch_issues_by_team.graphql";
const LINEAR_GET_WORKFLOW_STATES_BY_TEAM: &str = "queries/linear/get_workflow_states_by_team.graphql";
const LINEAR_UPDATE_ISSUE_WORKFLOW_STATE: &str = "queries/linear/update_issue_workflow_state.graphql";

pub async fn get_viewer(api_key: &str) -> Result<Value, GraphQLRequestError> {


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


pub async fn get_teams(api_key: &str) -> Result<Value, GraphQLRequestError> {


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


pub async fn get_issues_by_team(api_key: &str, variables: serde_json::Map<String, serde_json::Value>) -> Result<Value, GraphQLRequestError> {
    let mut query;
    query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_TEAM_PATH)?;

    query["variables"] = serde_json::Value::Object(variables);

    /*
    let resp: serde_json::Value = ureq::post("https://api.linear.app/graphql")
                                        .set("Content-Type", "application/json")
                                        .set("Authorization", api_key)
                                        .send_json(query)?
                                        .into_json()?;
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


pub async fn get_workflow_states_by_team(api_key: &str, variables: serde_json::Map<String, serde_json::Value>) -> Result<Value, GraphQLRequestError> {

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

pub async fn update_issue_workflow_state(api_key: &str, variables: serde_json::Map<String, serde_json::Value>) -> Result<Value, GraphQLRequestError> {

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