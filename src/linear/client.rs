use super::config::LinearConfig;

// Custom View Resolver Queries
use super::query::fetch_custom_views;
use super::query::fetch_issues_by_workflow_state as exec_fetch_issues_by_workflow_state;
use super::query::fetch_issues_by_assignee as exec_fetch_issues_by_assignee;

// Non Custom View Resolver Queries
use super::query::get_teams as exec_get_teams;
use super::query::get_issues_by_team as exec_get_issues_by_team;
use super::query::get_workflow_states_by_team as exec_get_workflow_states_by_team;
use super::query::update_issue_workflow_state as exec_update_issue_workflow_state;

use std::result::Result;

use super::error::LinearClientError;

use serde_json::Value;
use serde_json::json;

use crate::errors::*;

use crate::util::GraphQLCursor;

pub struct LinearClient {
    pub config: LinearConfig,
}

impl Default for LinearClient {
    fn default() -> LinearClient {
        LinearClient { config: LinearConfig::default() }
    }
}

impl LinearClient {

    fn set_config(&mut self, new_config: LinearConfig) {
        self.config = new_config;
    }

    // type LinearClientResult<T> = Result<T, LinearClientError>;

    pub async fn get_custom_views(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>) -> Result<Value, LinearClientError> {

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let query_response = fetch_custom_views(linear_api_key, linear_cursor, linear_config.issue_page_size).await?;

        let ref view_nodes = query_response["data"]["customViews"]["nodes"];
        let ref cursor_info = query_response["data"]["customViews"]["pageInfo"];


        Ok( json!( { "view_nodes": view_nodes.clone(), "cursor_info": cursor_info.clone() } ))
    }


    pub async fn get_teams(api_key: Option<String>) -> Result<serde_json::Value, LinearClientError> {

        let linear_api_key;

        info!("self.config.api_key: {:?}", api_key);

        match &api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };

        info!("linear_api_key: {:?}", linear_api_key);


        let query_response = exec_get_teams(linear_api_key).await?;

        let ref team_nodes = query_response["data"]["teams"]["nodes"];

        Ok(team_nodes.clone())
    }

    pub async fn get_issues_by_workflow_state( linear_config: LinearConfig, variables: serde_json::Map<String, serde_json::Value> ) -> Result<serde_json::Value, LinearClientError> {

        info!("Calling exec_fetch_issues_by_workflow_state - variables: {:?}", variables);

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let query_response = exec_fetch_issues_by_workflow_state(linear_api_key, variables).await?;

        let ref issue_nodes = query_response["data"]["workflowState"]["issues"]["nodes"];

        return Ok(issue_nodes.clone());

    }

    pub async fn get_issues_by_assignee( linear_config: LinearConfig, variables: serde_json::Map<String, serde_json::Value> ) -> Result<serde_json::Value, LinearClientError> {
        info!("Calling exec_fetch_issues_by_assignee - variables: {:?}", variables);

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let query_response = exec_fetch_issues_by_assignee(linear_api_key, variables).await?;

        let ref issue_nodes = query_response["data"]["user"]["assignedIssues"]["nodes"];

        return Ok(issue_nodes.clone());
    }


    pub async fn get_issues_by_team( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: serde_json::Map<String, serde_json::Value>) -> Result<serde_json::Value, LinearClientError> {

        info!("Calling exec_get_issues_by_team - variables: {:?}", variables);

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let query_response = exec_get_issues_by_team(linear_api_key, linear_cursor, linear_config.issue_page_size, variables).await?;

        let ref issue_nodes = query_response["data"]["team"]["issues"]["nodes"];
        let ref cursor_info = query_response["data"]["team"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes.clone(), "cursor_info": cursor_info.clone() } ))
        // Ok(issue_nodes.clone())
    }

    pub async fn get_workflow_states_by_team(api_key: Option<String>, variables: serde_json::Map<String, serde_json::Value>) -> Result<serde_json::Value, LinearClientError> {

        let linear_api_key;

        info!("Calling exec_get_workflow_states_by_team - variables: {:?}", variables);

        info!("self.config.api_key: {:?}", api_key);

        match &api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };

        info!("linear_api_key: {:?}", linear_api_key);


        let query_response = exec_get_workflow_states_by_team(linear_api_key, variables).await?;

        info!("get_workflow_states_by_team query_response: {:?}", query_response);

        let ref workflow_state_nodes = query_response["data"]["team"]["states"]["nodes"];

        Ok(workflow_state_nodes.clone())

    }
    // Note: This operation does not return a different response even if trying to set the Issue's workflow state to its current workflow state
    pub async fn update_issue_workflow_state(api_key: Option<String>, variables: serde_json::Map<String, serde_json::Value>) -> Result<serde_json::Value, LinearClientError> {

        info!("Calling update_issue_workflow_state - variables: {:?}", variables);

        let linear_api_key;
        match &api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };

        // info!("update_issue_workflow_state variables: {:?}", variables);

        let query_response = exec_update_issue_workflow_state(linear_api_key, variables).await?;

        // Response:
        /*
            {
                "data": {
                    "issueUpdate": {
                        "success": true,
                        "issue": {
                            "id": "ca14857a-b88e-4108-b97d-1c759358b9e7",
                            "title": "Test Rust-CLI 1",
                            "createdAt": "2021-02-09T20:05:52.185Z",
                            "number": 12
                        }
                    }
                }
            }
        */

        info!("update_issue_workflow_state query_response: {:?}", query_response);

        let ref issue_node = query_response["data"]["issueUpdate"]["issue"];
        let ref success = query_response["data"]["issueUpdate"]["success"];

        Ok( json!( { "issue_response": issue_node.clone(), "success": success.clone() } ) )

    }



}