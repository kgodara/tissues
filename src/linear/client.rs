use super::config::LinearConfig;

//Timezone query


// Custom View Resolver Queries
use super::query::fetch_custom_views;
use super::query::fetch_team_timezones as exec_fetch_team_timezones;

use super::query::fetch_all_issues as exec_fetch_all_issues;
use super::query::fetch_issues_by_team as exec_fetch_issues_by_team;
use super::query::fetch_issues_by_workflow_state as exec_fetch_issues_by_workflow_state;
use super::query::fetch_issues_by_assignee as exec_fetch_issues_by_assignee;
use super::query::fetch_issues_by_label as exec_fetch_issues_by_label;
use super::query::fetch_issues_by_creator as exec_fetch_issues_by_creator;
use super::query::fetch_issues_by_project as exec_fetch_issues_by_project;

// Non Custom View Resolver Queries
use super::query::get_teams as exec_get_teams;
use super::query::get_issues_by_team as exec_get_issues_by_team;
use super::query::get_workflow_states_by_team as exec_get_workflow_states_by_team;
use super::query::update_issue_workflow_state as exec_update_issue_workflow_state;

use std::result::Result;

use super::error::LinearClientError;

use serde_json::{ Value, Map};
use serde_json::json;

use crate::errors::*;

use crate::util::GraphQLCursor;

type ClientResult = Result<Value, LinearClientError>;

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

    pub async fn get_custom_views(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>) -> ClientResult {

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let query_response = fetch_custom_views(linear_api_key, linear_cursor, linear_config.custom_view_page_size).await?;

        let ref view_nodes = query_response["data"]["customViews"]["nodes"];
        let ref cursor_info = query_response["data"]["customViews"]["pageInfo"];


        Ok( json!( { "view_nodes": view_nodes.clone(), "cursor_info": cursor_info.clone() } ))
    }


    pub async fn get_teams(api_key: Option<String>) -> ClientResult {

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

    pub async fn fetch_team_timezones(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>) -> ClientResult {
        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let query_response = exec_fetch_team_timezones(linear_api_key, linear_cursor, linear_config.team_timezone_page_size).await?;

        let ref team_nodes = query_response["data"]["teams"]["nodes"];
        let ref cursor_info = query_response["data"]["teams"]["pageInfo"];


        Ok( json!( { "team_nodes": team_nodes.clone(), "cursor_info": cursor_info.clone() } ))
    }

    // View Resolver Query Section Start -------

    pub async fn get_all_issues( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, use_view_panel_config: bool ) -> ClientResult {

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let page_size: u32;
        if use_view_panel_config == true {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_all_issues(linear_api_key, linear_cursor, page_size).await?;

        let ref issue_nodes = query_response["data"]["issues"]["nodes"];
        let ref cursor_info = query_response["data"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes.clone(), "cursor_info": cursor_info.clone() } ))
    }

    pub async fn get_issues_by_team( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {

        info!("Calling exec_fetch_issues_by_team - variables: {:?}", variables);

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let page_size: u32;
        if use_view_panel_config == true {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issues_by_team(linear_api_key, linear_cursor, variables, page_size).await?;

        let ref issue_nodes = query_response["data"]["team"]["issues"]["nodes"];
        let ref cursor_info = query_response["data"]["team"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes.clone(), "cursor_info": cursor_info.clone() } ))
    }


    pub async fn get_issues_by_workflow_state( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {

        info!("Calling exec_fetch_issues_by_workflow_state - variables: {:?}", variables);

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let page_size: u32;
        if use_view_panel_config == true {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issues_by_workflow_state(linear_api_key, linear_cursor, variables, page_size).await?;

        let ref issue_nodes = query_response["data"]["workflowState"]["issues"]["nodes"];
        let ref cursor_info = query_response["data"]["workflowState"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes.clone(), "cursor_info": cursor_info.clone() } ))
    }

    pub async fn get_issues_by_assignee( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {
        info!("Calling exec_fetch_issues_by_assignee - variables: {:?}", variables);

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let page_size: u32;
        if use_view_panel_config == true {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issues_by_assignee(linear_api_key, linear_cursor, variables, page_size).await?;

        let ref issue_nodes = query_response["data"]["user"]["assignedIssues"]["nodes"];
        let ref cursor_info = query_response["data"]["user"]["assignedIssues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes.clone(), "cursor_info": cursor_info.clone() } ))

    }

    pub async fn get_issues_by_label( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {
        info!("Calling exec_fetch_issues_by_label - variables: {:?}", variables);

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let page_size: u32;
        if use_view_panel_config == true {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issues_by_label(linear_api_key, linear_cursor, variables, page_size).await?;

        let ref issue_nodes = query_response["data"]["issueLabel"]["issues"]["nodes"];
        let ref cursor_info = query_response["data"]["issueLabel"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes.clone(), "cursor_info": cursor_info.clone() } ))
    }

    pub async fn get_issues_by_creator( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool) -> ClientResult {
        info!("Calling exec_fetch_issues_by_creator - variables: {:?}", variables);

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };

        let page_size: u32;
        if use_view_panel_config == true {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }


        let query_response = exec_fetch_issues_by_creator(linear_api_key, linear_cursor, variables, page_size).await?;

        let ref issue_nodes = query_response["data"]["user"]["createdIssues"]["nodes"];
        let ref cursor_info = query_response["data"]["user"]["createdIssues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes.clone(), "cursor_info": cursor_info.clone() } ))
    }

    pub async fn get_issues_by_project( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool) -> ClientResult {
        info!("Calling exec_fetch_issues_by_assignee - variables: {:?}", variables);

        let linear_api_key;
        match &linear_config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let page_size: u32;
        if use_view_panel_config == true {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issues_by_project(linear_api_key, linear_cursor, variables, page_size).await?;

        let ref issue_nodes = query_response["data"]["project"]["issues"]["nodes"];
        let ref cursor_info = query_response["data"]["project"]["issues"]["pageInfo"];

        // debug!("get_issues_by_project issue_nodes: {:?}", issue_nodes);


        Ok( json!( { "issue_nodes": issue_nodes.clone(), "cursor_info": cursor_info.clone() } ))
    }

    // View Resolver Query Section End -------



    pub async fn get_issues_by_team_old( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>) -> ClientResult {

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

    pub async fn get_workflow_states_by_team(api_key: Option<String>, variables: Map<String, Value>) -> ClientResult {

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
    pub async fn update_issue_workflow_state(api_key: Option<String>, variables: Map<String, Value>) -> ClientResult {

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