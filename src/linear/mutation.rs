use crate::errors::{
    GraphQLRequestError
};
use crate::graphql::parse_graphql_from_file;

const LINEAR_CREATE_ISSUE_PATH: &str = "queries/linear/create_issue.graphql";


pub async fn create_linear_issue(api_key: &str, variables: serde_json::Map<String, serde_json::Value>) -> std::result::Result<serde_json::Value, GraphQLRequestError> {

    let mut mutation;
    mutation = parse_graphql_from_file(&LINEAR_CREATE_ISSUE_PATH)?;

    // println!("Test serde_json::Map.to_string: {}", serde_json::Value::Object(test_variables).to_string());

    mutation["variables"] = serde_json::Value::Object(variables); //serde_json::value::Value::String(mutation_variables); // serde_json::value::Value::String(mutation_variables);

    println!("Final Query: {}", mutation.to_string());

    

    /*
    let resp: serde_json::Value = ureq::post("https://api.linear.app/graphql")
                                        .set("Content-Type", "application/json")
                                        .set("Authorization", api_key)
                                        .send_json(mutation)?
                                        .into_json()?;
    */

    let client = reqwest::Client::new();

    let resp = client.post("https://api.linear.app/graphql")
                        .header("Content-Type", "application/json")
                        .header("Authorization", api_key)
                        .json(&mutation)
                        .send()
                        .await?
                        .json()
                        .await?;


    Ok(resp)
}