use serde_json::{ Value, Map };

use crate::linear::{
    LinearConfig,
    client::LinearClient,
};

use crate::app::Platform;
use crate::linear::{error::LinearClientError, types::CustomView };
use crate::util::GraphQLCursor;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum FilterType {

    SelectedTeam,
    AllTeams,

    // Only one Content filter per view
    Content,

    SelectedState,
    SelectedCreator,
    SelectedLabel,
    SelectedAssignee,
    SelectedProject,
    SelectedPriority,
    SelectedSubscriber,

    DueToday,
    Overdue,
    HasDueDate,
    DueSoon,
    NoDueDate,

    NoLabel,
    NoAssignee,
    NoProject,
}

pub async fn single_endpoint_fetch (  view_cursor: &mut GraphQLCursor,
    request_num: &mut u32,
    filter_data: &mut Value,
    linear_config: &LinearConfig,
) -> Vec<Value> {

    let mut found_issue_list: Vec<Value> = Vec::new();

    let mut loop_num: u16 = 0;

    let mut query_result: Result<Value, LinearClientError>;
    let mut variables: Map<String, Value> = Map::new();

    loop {

        // If Query is exhausted
        if view_cursor.platform == Platform::Linear && !view_cursor.has_next_page {
            // No more Pages remaining, return found_issues_list
            debug!("Single Endpoint Fetch - no more issues to query, returning found_issues_list");
            return found_issue_list;
        }

        variables.insert(String::from("filterObj"), filter_data.clone());

        query_result = LinearClient::get_issues_by_filter_data(linear_config.clone(), Some(view_cursor.clone()), variables.clone()).await;

        if let Ok(response) = query_result {

            // Increment request_num here
            *request_num += 1;

            debug!("Current Filter Data Query Response: {:?}", response);

            // Filter returned Issues by all other loader filters
            // and add remainder to final_issue_list

            let mut returned_issues: Vec<Value> = match response["issue_nodes"].as_array() {
                Some(resp_issue_data) => {
                    resp_issue_data.clone()
                },
                None => {
                    error!("'issue_nodes' invalid format: {:?}", response["issue_nodes"]);
                    panic!("'issue_nodes' invalid format");
                }
            };

            debug!("returned_issues.len(): {:?}", returned_issues.len());

            if !returned_issues.is_empty() {
                found_issue_list.append(&mut returned_issues);
            }

            // Update GraphQLCursor
            match GraphQLCursor::linear_cursor_from_page_info(response["cursor_info"].clone()) {
                Some(new_cursor) => {
                    *view_cursor = new_cursor.clone();
                },
                None => {
                    error!("GraphQLCursor could not be created from response['cursor_info']: {:?}", response["cursor_info"]);
                    panic!("GraphQLCursor could not be created from response['cursor_info']: {:?}", response["cursor_info"]);
                },
            }
        }
        else {
            error!("View_Resolver Query Failed: {:?}", query_result);
            panic!("View_Resolver Query Failed: {:?}", query_result);
        }

        if found_issue_list.len() >= (linear_config.view_panel_page_size as usize)  {
            return found_issue_list;
        }

        info!("Loop {} - found_issue_list: {:?}", loop_num, found_issue_list);
        loop_num += 1;
    }
}

pub async fn optimized_view_issue_fetch (   view_obj: &CustomView,
                                            cursor_option: Option<GraphQLCursor>,
                                            linear_config: LinearConfig
                                        ) -> ( Vec<Value>, GraphQLCursor, u32 ) {

    info!("View Resolver received view_obj: {:?}", view_obj);

    let mut filter_data = view_obj.filter_data.clone();

    let mut view_cursor =  if let Some(cursor) = cursor_option { cursor } else { GraphQLCursor::default() };

    debug!("View Cursor: {:?}", view_cursor);


    let mut request_num: u32 = 0;
    let found_issue_list: Vec<Value> = single_endpoint_fetch(
        &mut view_cursor, &mut request_num,
        &mut filter_data, &linear_config).await;


    info!("'optimized_view_issue_fetch' returning found_issue_list.len(): {:?}", found_issue_list.len());
    info!("'optimized_view_issue_fetch' returning found_issue_list: {:?}", found_issue_list);

    (found_issue_list, view_cursor, request_num)
}