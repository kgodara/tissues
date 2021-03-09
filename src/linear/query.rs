use crate::graphql::{
    parse_graphql_from_file,
};

use crate::errors::{
    GraphQLError,
};

use serde_json::{
    Value
};

use std::result::Result;

const LINEAR_GET_VIEWER_PATH: &str = "queries/linear/get_viewer.graphql";
const LINEAR_GET_TEAMS_PATH: &str = "queries/linear/get_teams.graphql";
const LINEAR_FETCH_ISSUES_BY_TEAM_PATH: &str = "queries/linear/fetch_issues_by_team.graphql";

pub fn get_viewer(api_key: &str) -> Result<Value, GraphQLError> {


    let query;
    query = parse_graphql_from_file(&LINEAR_GET_VIEWER_PATH)?;


    // Requires the `json` feature enabled.
    let resp: Value = ureq::post("https://api.linear.app/graphql")
                            .set("Content-Type", "application/json")
                            .set("Authorization", api_key)
                            .send_json(query)?
                            .into_json()?;
                            /*.into_string()?;*/

    Ok(resp)

}


pub fn get_teams(api_key: &str) -> Result<Value, GraphQLError> {


    let query;
    query = parse_graphql_from_file(&LINEAR_GET_TEAMS_PATH)?;


    // Requires the `json` feature enabled.
    let resp: Value = ureq::post("https://api.linear.app/graphql")
                            .set("Content-Type", "application/json")
                            .set("Authorization", api_key)
                            .send_json(query)?
                            .into_json()?;
                            /*.into_string()?;*/

    Ok(resp)

}


pub fn get_issues_by_team(api_key: &str, variables: serde_json::Map<String, serde_json::Value>) -> Result<Value, GraphQLError> {
    let mut query;
    query = parse_graphql_from_file(&LINEAR_FETCH_ISSUES_BY_TEAM_PATH)?;

    query["variables"] = serde_json::Value::Object(variables);

    let resp: serde_json::Value = ureq::post("https://api.linear.app/graphql")
                                        .set("Content-Type", "application/json")
                                        .set("Authorization", api_key)
                                        .send_json(query)?
                                        .into_json()?;

    Ok(resp)

}