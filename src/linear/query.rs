use crate::graphql;

const LINEAR_GET_VIEWER_PATH: &str = "queries/linear/get_viewer.graphql";
const LINEAR_GET_TEAMS_PATH: &str = "queries/linear/get_teams.graphql";

pub fn get_linear_viewer(api_key: &str) -> std::result::Result<serde_json::Value, graphql::GraphQLError> {


    let query;
    query = graphql::parse_graphql_from_file(&LINEAR_GET_VIEWER_PATH)?;


    // Requires the `json` feature enabled.
    let resp: serde_json::Value = ureq::post("https://api.linear.app/graphql")
                            .set("Content-Type", "application/json")
                            .set("Authorization", api_key)
                            .send_json(query)?
                            .into_json()?;
                            /*.into_string()?;*/

    Ok(resp)

}


pub fn get_linear_teams(api_key: &str) -> std::result::Result<serde_json::Value, graphql::GraphQLError> {


    let query;
    query = graphql::parse_graphql_from_file(&LINEAR_GET_TEAMS_PATH)?;


    // Requires the `json` feature enabled.
    let resp: serde_json::Value = ureq::post("https://api.linear.app/graphql")
                            .set("Content-Type", "application/json")
                            .set("Authorization", api_key)
                            .send_json(query)?
                            .into_json()?;
                            /*.into_string()?;*/

    Ok(resp)

}