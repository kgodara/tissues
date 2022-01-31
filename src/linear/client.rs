use super::config::{ LinearConfig, MAX_PAGE_SIZE };

// Timezone query


// Custom View Resolver Queries

use super::query::{
    exec_fetch_custom_views,
    exec_fetch_team_timezones,
    exec_fetch_viewer,
    exec_fetch_workflow_states,

    exec_fetch_all_issues,
    exec_fetch_issues_by_team,

    exec_fetch_issue_by_direct_filter,
    exec_fetch_issue_single_endpoint,

    // Non Custom View Resolver Queries
    // exec_get_teams,

    exec_get_issue_op_data,
    exec_update_issue,
};


use std::result::Result;

use std::sync::{ Arc, Mutex };

use super::error::LinearClientError;

use serde_json::{ Value, json, Map};

use crate::errors::ConfigError;

use crate::util::{
    GraphQLCursor,
    verify_linear_api_key_present
};

use crate::linear::view_resolver::FilterType;

use crate::constants::{
    IssueModificationOp
};


pub type ClientResult = Result<Value, LinearClientError>;

pub struct LinearClient {
    pub config: Arc<Mutex<LinearConfig>>,
}

impl Default for LinearClient {
    fn default() -> LinearClient {
        LinearClient { config: Arc::new(Mutex::new(LinearConfig::default())) }
    }
}

impl LinearClient {

    /*
    fn set_config(&mut self, new_config: LinearConfig) {
        self.config = new_config;
    }
    */

    // type LinearClientResult<T> = Result<T, LinearClientError>;

    pub async fn get_custom_views(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>) -> ClientResult {

        let linear_api_key = &verify_linear_api_key_present(&linear_config)?;

        let query_response = exec_fetch_custom_views(&linear_api_key, linear_cursor, linear_config.custom_view_page_size).await?;

        let view_nodes = &query_response["data"]["customViews"]["nodes"];
        let cursor_info = &query_response["data"]["customViews"]["pageInfo"];


        Ok( json!( { "view_nodes": view_nodes, "cursor_info": cursor_info } ))
    }

    pub async fn fetch_team_timezones(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>) -> ClientResult {

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let query_response = exec_fetch_team_timezones(&linear_api_key, linear_cursor, linear_config.team_timezone_page_size).await?;

        let team_nodes = &query_response["data"]["teams"]["nodes"];
        let cursor_info = &query_response["data"]["teams"]["pageInfo"];


        Ok( json!( { "team_nodes": team_nodes, "cursor_info": cursor_info } ))
    }

    // api_key here since hasn't been set to LinearConfig yet
    pub async fn fetch_viewer(api_key: &str) -> ClientResult {

        let query_response = exec_fetch_viewer(api_key).await?;

        let viewer_node = &query_response["data"]["viewer"];
        let error_node = &query_response["errors"];


        Ok( json!( { "viewer_node": viewer_node, "error_node": error_node } ))
    }

    pub async fn get_issues_by_filter_data(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>) -> ClientResult {

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let query_response = exec_fetch_issue_single_endpoint(&linear_api_key, linear_cursor, variables, linear_config.issue_page_size).await?;

        let issue_nodes = &query_response["data"]["issues"]["nodes"];
        let cursor_info = &query_response["data"]["issues"]["pageInfo"];

        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))

    }

    // view_resolver 'filter_data' support query (workflow state name case-sensitivity)
    pub async fn fetch_workflow_states(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>) -> ClientResult {
        debug!("fetch_workflow_states initalized");

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let query_response = exec_fetch_workflow_states(&linear_api_key, linear_cursor, MAX_PAGE_SIZE).await?;

        let state_nodes = &query_response["data"]["workflowStates"]["nodes"];
        let cursor_info = &query_response["data"]["workflowStates"]["pageInfo"];

        Ok( json!( { "state_nodes": state_nodes, "cursor_info": cursor_info } ))

    }

    // View Resolver Query Section Start -------

    // generic_issue_fetch Section Start -------
    pub async fn get_all_issues( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, use_view_panel_config: bool ) -> ClientResult {

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;


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

    // Use for generic_issue_fetch
    pub async fn get_issues_by_team( linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {

        info!("Calling exec_fetch_issues_by_team - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;


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
    // generic_issue_fetch Section End -------


    // direct_issue_fetch Section Start -------
    pub async fn get_issues_by_content(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {
        info!("Calling exec_fetch_issues_by_content - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }

        let query_response = exec_fetch_issue_by_direct_filter(&FilterType::Content, &linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["issueSearch"]["nodes"];
        let cursor_info = &query_response["data"]["issueSearch"]["pageInfo"];

        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))
    }


    pub async fn get_issues_by_workflow_state(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {

        info!("Calling exec_fetch_issues_by_workflow_state - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issue_by_direct_filter(&FilterType::SelectedState, &linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["workflowState"]["issues"]["nodes"];
        let cursor_info = &query_response["data"]["workflowState"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))
    }

    pub async fn get_issues_by_assignee(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {
        info!("Calling exec_fetch_issues_by_assignee - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issue_by_direct_filter(&FilterType::SelectedAssignee, &linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["user"]["assignedIssues"]["nodes"];
        let cursor_info = &query_response["data"]["user"]["assignedIssues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))

    }

    pub async fn get_issues_by_label(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool ) -> ClientResult {
        info!("Calling exec_fetch_issues_by_label - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issue_by_direct_filter(&FilterType::SelectedLabel, &linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["issueLabel"]["issues"]["nodes"];
        let cursor_info = &query_response["data"]["issueLabel"]["issues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))
    }

    pub async fn get_issues_by_creator(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool) -> ClientResult {
        info!("Calling exec_fetch_issues_by_creator - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }


        let query_response = exec_fetch_issue_by_direct_filter(&FilterType::SelectedCreator, &linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["user"]["createdIssues"]["nodes"];
        let cursor_info = &query_response["data"]["user"]["createdIssues"]["pageInfo"];


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))
    }

    pub async fn get_issues_by_project(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>, use_view_panel_config: bool) -> ClientResult {
        info!("Calling exec_fetch_issues_by_assignee - variables: {:?}", variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;


        let page_size: u32;
        if use_view_panel_config {
            page_size = linear_config.issue_page_size;
        }
        else {
            page_size = linear_config.view_panel_page_size;
        }



        let query_response = exec_fetch_issue_by_direct_filter(&FilterType::SelectedProject, &linear_api_key, linear_cursor, variables, page_size).await?;

        let issue_nodes = &query_response["data"]["project"]["issues"]["nodes"];
        let cursor_info = &query_response["data"]["project"]["issues"]["pageInfo"];

        // debug!("get_issues_by_project issue_nodes: {:?}", issue_nodes);


        Ok( json!( { "issue_nodes": issue_nodes, "cursor_info": cursor_info } ))
    }
    // direct_issue_fetch Section End -------

    // View Resolver Query Section End -------

    // Issue Modification Queries

    pub async fn get_workflow_states_by_team(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>) -> ClientResult {

        info!("Calling exec_get_issue_op_data - {:?} - variables: {:?}", IssueModificationOp::WorkflowState, variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let query_response = exec_get_issue_op_data(&IssueModificationOp::WorkflowState, &linear_api_key, linear_cursor, variables, linear_config.issue_op_page_size).await?;

        let workflow_state_nodes = &query_response["data"]["team"]["states"]["nodes"];
        let cursor_info = &query_response["data"]["team"]["states"]["pageInfo"];

        Ok( json!( { "data_nodes": workflow_state_nodes, "cursor_info": cursor_info } ))
    }
    pub async fn get_users_by_team(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>) -> ClientResult {
        info!("Calling exec_get_issue_op_data - {:?} - variables: {:?}", IssueModificationOp::Assignee, variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let query_response = exec_get_issue_op_data(&IssueModificationOp::Assignee, &linear_api_key, linear_cursor, variables, linear_config.issue_op_page_size).await?;

        let user_nodes = &query_response["data"]["team"]["members"]["nodes"];
        let cursor_info = &query_response["data"]["team"]["members"]["pageInfo"];

        Ok( json!( { "data_nodes": user_nodes, "cursor_info": cursor_info } ))
    }
    pub async fn get_projects_by_team(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>) -> ClientResult {
        info!("Calling exec_get_issue_op_data - {:?} - variables: {:?}", IssueModificationOp::Project, variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let query_response = exec_get_issue_op_data(&IssueModificationOp::Project, &linear_api_key, linear_cursor, variables, linear_config.issue_op_page_size).await?;

        let project_nodes = &query_response["data"]["team"]["projects"]["nodes"];
        let cursor_info = &query_response["data"]["team"]["projects"]["pageInfo"];

        Ok( json!( { "data_nodes": project_nodes, "cursor_info": cursor_info } ))
    }
    pub async fn get_cycles_by_team(linear_config: LinearConfig, linear_cursor: Option<GraphQLCursor>, variables: Map<String, Value>) -> ClientResult {
        info!("Calling exec_get_issue_op_data - {:?} - variables: {:?}", IssueModificationOp::Cycle, variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let query_response = exec_get_issue_op_data(&IssueModificationOp::Cycle, &linear_api_key, linear_cursor, variables, linear_config.issue_op_page_size).await?;

        let cycle_nodes = &query_response["data"]["team"]["cycles"]["nodes"];
        let cursor_info = &query_response["data"]["team"]["cycles"]["pageInfo"];

        Ok( json!( { "data_nodes": cycle_nodes, "cursor_info": cursor_info } ))
    }

    // Note: These operations generally don't return a different response even if trying to set a property to the currently set value
    pub async fn update_issue(op: &IssueModificationOp, linear_config: LinearConfig, variables: Map<String, Value>) -> ClientResult {
        debug!("update_issue - {:?} - variables: {:?}", op, variables);

        let linear_api_key = verify_linear_api_key_present(&linear_config)?;

        let query_response = exec_update_issue(op, &linear_api_key, variables).await?;

        let issue_node = &query_response["data"]["issueUpdate"]["issue"];
        let success = &query_response["data"]["issueUpdate"]["success"];

        Ok( json!( { "issue_response": issue_node, "success": success } ) )
    }
}