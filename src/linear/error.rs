use thiserror::Error;
use crate::errors;

#[derive(Error, Debug)]
pub enum LinearClientError {
    
    #[error("Invalid configuration")]
    InvalidConfig(#[from] errors::ConfigError),
    #[error("GraphQL failure")]
    GraphQL(#[from] errors::GraphQLError),

    #[error("GraphQL request failure")]
    RequestError(#[from] errors::GraphQLRequestError)
}