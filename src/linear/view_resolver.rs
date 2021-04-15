
use serde_json::Value;
use crate::linear::LinearConfig;
use crate::linear::client::LinearClient;

use crate::errors::ConfigError;
use crate::errors::GraphQLRequestError;
use super::error::LinearClientError;

use std::sync::{ Arc, Mutex };


// Accepts a custom view object, in format: serde_json::Value
pub async fn get_issues_from_view( view_obj: &Value, linear_config: LinearConfig ) -> Option<Value> {
    // Determine which filter resolver methods to call based on available attributes,
    // and perform an Intersect operation on returned issues to determine 
    
    info!("View Resolver received view_obj: {:?}", view_obj);

    let filters = view_obj["filters"].clone();

    match filters {
        Value::Object(_) => {
            let filter_component_results: Vec<Option<Vec<Value>>>;

            // Handle 'state' filters
            match filters["state"].as_array() {
                Some(state_list) => {
                    get_issues_by_state_filters(state_list.clone(), linear_config.clone()).await;
                },
                _ => {},
            };

        },
        Value::Null => { return None },
        _ => { panic!("view.filters was not an Object or Null") },
    };

    return None;
}

// Is it better to fetch all issues by workflow states, then filter by state_list?
// Current approach: query once for each workflow state present in state_list, then merge
// wofklowStates() -> { nodes [ { issues() } ] }
async fn get_issues_by_state_filters( state_list: Vec<Value>, linear_config: LinearConfig ) -> Option<Vec<Value>> {
    info!("get_issues_by_state_filters received state_list: {:?}", state_list);
    // note the use of `into_iter()` to consume `items`
    let tasks: Vec<_> = state_list
    .into_iter()
    .map(|mut item| {
        info!("Spawning Get Issue By Workflow State Task");
        let temp_config = linear_config.clone();
        tokio::spawn(async move {
            match item.as_object() {
                Some(state_obj) => {
                    let state_issues = LinearClient::get_issues_by_workflow_state( temp_config, state_obj.clone() ).await;
                    state_issues
                },
                _ => {
                    Err( LinearClientError::InvalidConfig( ConfigError::InvalidParameter { parameter: String::from("Workflow State Obj not found") } ) )
                },
            }
        })
    })
    .collect();

    // await the tasks for resolve's to complete and give back our items
    let mut items = vec![];
    for task in tasks {
        items.push(task.await.unwrap());
    }
    // verify that we've got the results
    for item in &items {
        info!("get_issues_by_workflow_state Result: {:?}", item);
    }

    let issues: Vec<Value> = items
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    })
                    .collect();
    info!("get_issues_by_workflow_state Issues: {:?}", issues);


    return Some(issues);
}

// Note: Currently will ignore 'No Assignee' filter
async fn get_issues_by_assignee ( assignee_list: Vec<Value>, linear_config: LinearConfig ) -> Option<Vec<Value>> {
    info!("get_issues_by_assignee received assignee_list: {:?}", assignee_list);

    // note the use of `into_iter()` to consume `items`
    let tasks: Vec<_> = assignee_list
    .into_iter()
    .map(|mut item| {

        let mut invalid_filter = false;

        // If 'item' does not have a ref, is 'No Assignee' filter, skip
        if let Value::Null = item["ref"] {
            invalid_filter = true;
        }

        info!("Spawning Get Issue By Assignee Task");
        let temp_config = linear_config.clone();
        tokio::spawn(async move {
            if invalid_filter == true {
                return Err(LinearClientError::InvalidConfig(
                        ConfigError::InvalidParameter { parameter: String::from("View Assignee filter") }
                    )
                );
            };
            match item.as_object() {
                Some(assignee_obj) => {
                    let assignee_issues = LinearClient::get_issues_by_assignee( temp_config, assignee_obj.clone() ).await;
                    assignee_issues
                },
                _ => {
                    Err( LinearClientError::InvalidConfig( ConfigError::InvalidParameter { parameter: String::from("Assignee Obj not found") } ) )
                },
            }
        })
    })
    .collect();

    // await the tasks for resolve's to complete and give back our items
    let mut items = vec![];
    for task in tasks {
        items.push(task.await.unwrap());
    }
    // verify that we've got the results
    for item in &items {
        info!("get_issues_by_assignee Result: {:?}", item);
    }

    let issues: Vec<Value> = items
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    })
                    .collect();
    info!("get_issues_by_assignee Issues: {:?}", issues);


    return Some(issues);

}

async fn get_issues_by_creator ( creator_list:  Vec<Value>, linear_config: LinearConfig ) -> Option<Vec<Value>> {
    info!("get_issues_by_creator received creator: {:?}", creator_list);

    // note the use of `into_iter()` to consume `items`
    let tasks: Vec<_> = creator_list
    .into_iter()
    .map(|mut item| {

        info!("Spawning Get Issue By Creator Task");
        let temp_config = linear_config.clone();
        tokio::spawn(async move {
            match item.as_object() {
                Some(creator_obj) => {
                    let creator_issues = LinearClient::get_issues_by_creator( temp_config, creator_obj.clone() ).await;
                    creator_issues
                },
                _ => {
                    Err( LinearClientError::InvalidConfig( ConfigError::InvalidParameter { parameter: String::from("Creator Obj not found") } ) )
                },
            }
        })
    })
    .collect();

    // await the tasks for resolve's to complete and give back our items
    let mut items = vec![];
    for task in tasks {
        items.push(task.await.unwrap());
    }
    // verify that we've got the results
    for item in &items {
        info!("get_issues_by_creator Result: {:?}", item);
    }

    let issues: Vec<Value> = items
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    })
                    .collect();
    info!("get_issues_by_creator Issues: {:?}", issues);


    return Some(issues);
}