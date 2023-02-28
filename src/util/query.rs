use serde_json::Value;

use crate::util::GraphQLCursor;
use crate::app::Platform;

use crate::linear::{
    LinearConfig,
    error::LinearClientError,
};

use crate::errors::{
    GraphQLRequestError,
    ConfigError
};

pub fn set_linear_after_cursor_from_opt(variables: &mut Value, cursor_opt: Option<GraphQLCursor>) -> Result<(), GraphQLRequestError> {

    if let Some(cursor_data) = cursor_opt {
        // If Cursor is for a different platform, and is not a new cursor
        if cursor_data.platform != Platform::Linear && cursor_data.platform != Platform::Na {
            return Err(GraphQLRequestError::InvalidCursor(cursor_data));
        }
        if cursor_data.has_next_page && cursor_data.platform == Platform::Linear {
            variables["afterCursor"] = Value::String(cursor_data.end_cursor);
        }
    }

    Ok(())
}

pub fn linear_after_from_opt(cursor_opt: &Option<GraphQLCursor>) -> Option<String> {

    if let Some(cursor_data) = cursor_opt {
        if cursor_data.platform == Platform::Linear && cursor_data.has_next_page {
            debug!("linear_after_from_opt returning Some()");
            Some(cursor_data.end_cursor.to_string())
        }
        else {
            None
        }
    } else {
        None
    }
}



pub fn verify_linear_api_key_present(linear_config: &LinearConfig) -> Result<String, LinearClientError> {
    match &linear_config.api_key {
        Some(x) => Ok(x.to_string()),
        None => Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
    }
}