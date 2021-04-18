use std::fs;

use crate::errors::GraphQLParseError;



pub fn parse_graphql_from_file(file_path: &str) -> std::result::Result<serde_json::Value, GraphQLParseError> {

    let mut query_contents = fs::read_to_string(file_path)?;

    
    query_contents = query_contents.as_str()
                                    .replace("\n", "");
    
    // Parse the string of data into serde_json::Value.
    let v: serde_json::Value = serde_json::from_str(query_contents.as_str())?;

    info!("Fetched query: {}", v["query"]);

    Ok(v)
}