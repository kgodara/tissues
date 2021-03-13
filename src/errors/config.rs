use thiserror::Error;
use std::io;


#[derive(Error, Debug)]
pub enum GraphQLParseError {
    #[error("Failed to read GraphQL query file")]
    FileReadFailure(#[from] io::Error),
    #[error("Failed to parse GraphQL query into JSON")]
    JSONParseFailure(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum GraphQLRequestError {
    #[error("GraphQL operation parse failed")]
    GraphQLParseFailure(#[from] GraphQLParseError),
    #[error("GraphQL request failed")]
    GraphQLRequestError(#[from] reqwest::Error)
}

#[derive(Error, Debug)]
pub enum ConfigError {
    /*
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    */
    /*
    #[error("invalid header (platform {platform:?}, found {found:?})")]
    InvalidHeader {
        platform: String,
        found: String,
    },
    */
    /*
    #[error("unknown data store error")]
    Unknown,
    */
    
    #[error("Credentials not found for ${platform:?}")]
    CredentialsNotFound {
        platform: String,
    },

}