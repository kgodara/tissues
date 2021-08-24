use super::config::LinearConfig;

//Timezone query


// Custom View Resolver Queries

use super::query::{
    exec_fetch_custom_views,
    exec_fetch_team_timezones,

    exec_fetch_all_issues,
    exec_fetch_issues_by_team,
    exec_fetch_issues_by_workflow_state,
    exec_fetch_issues_by_assignee,
    exec_fetch_issues_by_label,
    exec_fetch_issues_by_creator,
    exec_fetch_issues_by_project,

    // Non Custom View Resolver Queries
    exec_get_teams,
    exec_get_workflow_states_by_team,
    exec_update_issue_workflow_state,
};


use std::result::Result;

use super::error::LinearClientError;

use serde_json::{ Value, Map};
use serde_json::json;

use crate::errors::ConfigError;

use crate::util::GraphQLCursor;

use crate::util::verify_linear_api_key;
// use crate

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

        let linear_api_key = verify_linear_api_key(&linear_config)?;

        let query_response = exec_fetch_custom_views(&linear_api_key, linear_cursor, linear_config.custom_view_page_size).await?;

        let view_nodes = &query_response["data"]["customViews"]["nodes"];
        let cursor_info = &query_response["data"]["customViews"]["pageInfo"];


        Ok( json!( { "view_nodes": view_nodes, "cursor_info": cursor_info } ))
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

        let team_nodes = &query_response["data"]["teams"]["nodes"];

        Ok(team_nodes.clone())
    }

    pub async fn fetch_team_timezones(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>) -> ClientResult {

        let linear_api_key = verify_linear_api_key(&linear_config)?;

        let query_response = exec_fetch_team_timezones(&linear_api_key, linear_cursor, linear_config.team_timezone_page_size).await?;

        let team_nodes = &query_response["data"]["teams"]["nodes"];
        let cursor_info = &query_response["data"]["teams"]["pageInfo"];


        Ok( json!( { "team_nodes": team_nodes, "cursor_info": cursor_info } ))
    }

    // View Resolver Query Section Start -------

    pub async fn get_all_issues( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, use_view_panel_config: bool ) -> ClientResult {

        let linear_api_key = verify_linear_api_key(&linear_config)?;


        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }

        let query_response = exec_fetch_all_issues(&linear_api_key, linear_cursor, page_size).await?;

        let issue_nodes = &query_response["data"]["issues"]["nodes"];
        let cursor_info = &query_response["data"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))
    }

    pub async fn get_issues_by_team( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {

        info!("Calling exec_fetch_issues_by_team - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key(&linear_config)?;


        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }

        let query_response = exec_fetch_issues_by_team(&linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["team"]["issues"]["nodes"];
        let cursor_info = &query_response["data"]["team"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes.clone(), "cursor_info": cursor_info.clone() } ))
    }


    pub async fn get_issues_by_workflow_state( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {

        info!("Calling exec_fetch_issues_by_workflow_state - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key(&linear_config)?;

        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issues_by_workflow_state(&linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["workflowState"]["issues"]["nodes"];
        let cursor_info = &query_response["data"]["workflowState"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))
    }

    pub async fn get_issues_by_assignee( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {
        info!("Calling exec_fetch_issues_by_assignee - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key(&linear_config)?;

        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issues_by_assignee(&linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["user"]["assignedIssues"]["nodes"];
        let cursor_info = &query_response["data"]["user"]["assignedIssues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))

    }

    pub async fn get_issues_by_label( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {
        info!("Calling exec_fetch_issues_by_label - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key(&linear_config)?;

        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issues_by_label(&linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["issueLabel"]["issues"]["nodes"];
        let cursor_info = &query_response["data"]["issueLabel"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))
    }

    pub async fn get_issues_by_creator( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool) -> ClientResult {
        info!("Calling exec_fetch_issues_by_creator - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key(&linear_config)?;

        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }


        let query_response = exec_fetch_issues_by_creator(&linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["user"]["createdIssues"]["nodes"];
        let cursor_info = &query_response["data"]["user"]["createdIssues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))
    }

    pub async fn get_issues_by_project( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool) -> ClientResult {
        info!("Calling exec_fetch_issues_by_assignee - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key(&linear_config)?;


        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issues_by_project(&linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["project"]["issues"]["nodes"];
        let cursor_info = &query_response["data"]["project"]["issues"]["pageInfo"];

        // debug!("get_issues_by_project issue_nodes: {:?}", issue_nodes);


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))
    }

    // View Resolver Query Section End -------

    pub async fn get_workflow_states_by_team(linear_config: LinearConfig, variables: Map<String, Value>) -> ClientResult {

        info!("Calling exec_get_workflow_states_by_team - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key(&linear_config)?;

        let query_response = exec_get_workflow_states_by_team(&linear_api_key, variables).await?;

        let workflow_state_nodes = &query_response["data"]["team"]["states"]["nodes"];

        Ok(workflow_state_nodes.clone())
    }

    // Note: This operation does not return a different response even if trying to set the Issue's workflow state to its current workflow state
    pub async fn update_issue_workflow_state(linear_config: LinearConfig, variables: Map<String, Value>) -> ClientResult {

        info!("Calling update_issue_workflow_state - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key(&linear_config)?;

        let query_response = exec_update_issue_workflow_state(&linear_api_key, variables).await?;

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

        let issue_node = &query_response["data"]["issueUpdate"]["issue"];
        let success = &query_response["data"]["issueUpdate"]["success"];

        Ok( json!( { "issue_response": issue_node, "success": success } ) )

    }




}