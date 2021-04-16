
use serde_json::Value;
use crate::linear::LinearConfig;
use crate::linear::client::LinearClient;

use crate::errors::ConfigError;
use crate::errors::GraphQLRequestError;
use super::error::LinearClientError;

use std::sync::{ Arc, Mutex };

use std::collections::HashSet;


// Accepts a custom view object, in format: serde_json::Value
// TODO: Certain filter resolvers may have to paginate in order to create accurate view and filter lists
pub async fn get_issues_from_view( view_obj: &Value, linear_config: LinearConfig ) -> Option<Vec<Value>> {
    // Determine which filter resolver methods to call based on available attributes,
    // and perform an Intersect operation on returned issues to determine 
    
    info!("View Resolver received view_obj: {:?}", view_obj);

    let filters = view_obj["filters"].clone();

    if Value::Null == filters {
        return None;
    }

    if let Value::Object(_) = filters {

        let mut filter_component_results: Vec<Option<Vec<Value>>> = Vec::new();

        // Handle 'state' filters
        match filters["state"].as_array() {
            Some(state_list) => {
                filter_component_results.push(
                    get_issues_by_state(state_list.clone(), linear_config.clone()).await
                );
            },
            _ => {},
        };

        // Handle 'assignee' filters
        match filters["assignee"].as_array() {
            Some(assignee_list) => {
                filter_component_results.push(
                    get_issues_by_assignee(assignee_list.clone(), linear_config.clone()).await
                );
            }
            _ => {},
        }

        // Handle 'creator' filters
        match filters["creator"].as_array() {
            Some(creator_list) => {
                filter_component_results.push(
                    get_issues_by_creator(creator_list.clone(), linear_config.clone()).await
                );
            }
            _ => {},
        }
        
        // Final Merge operation:
        //     let mut final_issue_list: Vec<Value> = Vec::new();
        //     let mut issue_id_set: Set<String>
        //     Remove 'None' reponses from filter_component_results
        //     Iterate over all Some(Vec<Value>) responses in filter_component_results, for each:
        //         If first list, add all issue ids to issue_id_set & for every issue id not already in issue_id_set, add issue to final issue list
        //         Else:
        //             Create array of issue ids 'unfound_issue_ids' from issue_id_set
        //             iterate over current Vec<Value>
        //                 if issue id not in issue_id_set, skip
        //                 else remove issue id from 'unfound_issue_ids'
        //             Remove all issue_ids remaining in 'unfound_issue_ids' from 'issue_id_set'
        //             Remove all issues from 'final_issue_list' whose ids are in 'unfound_issue_ids'
        //    Return final_issue_list

        let mut final_issue_list: Vec<Value> = Vec::new();
        let mut issue_id_set: HashSet<String> = HashSet::new();

        let mut resolver_issue_values: Vec<Vec<Value>> = filter_component_results
                                    .into_iter()
                                    .filter_map(|e| {
                                        match e {
                                            Some(_) => e,
                                            None => None,
                                        }
                                    })
                                    .collect();
    
        for (idx, value_list) in resolver_issue_values.iter().enumerate() {

            // If first list, add all issue ids to issue_id_set 
            //     for every issue id not already in issue_id_set, add issue to final issue list

            if idx == 0 {
                let mut current_issue_id: String;
                for issue_obj in resolver_issue_values[idx].iter() {
                    let mut already_in_set = false;

                    match issue_obj["id"].as_str() {
                        Some(id) => {
                            already_in_set = issue_id_set.insert(String::from(id));
                        },
                        None => {continue;},
                    }
                    // Issue id not already in issue_id_set, add issue to final issue list
                    if already_in_set == false {
                        final_issue_list.push(issue_obj.clone());
                    }
                }
            }

            // Else:
            //     Create array of issue ids 'unfound_issue_ids' from issue_id_set
            //     iterate over current Vec<Value> 'value_list'
            //         if issue id not in issue_id_set, skip
            //         else remove issue id from 'unfound_issue_ids'
            //     Remove all issue_ids remaining in 'unfound_issue_ids' from 'issue_id_set'
            //     Remove all issues from 'final_issue_list' whose ids are in 'unfound_issue_ids'
            else {
                let mut unfound_issue_ids: HashSet<String> = issue_id_set.iter()
                                                                        .cloned()
                                                                        .collect();
                // iterate over current Vec<Value> 'value_list'
                for issue_obj in value_list.iter() {
                    match issue_obj["id"].as_str() {
                        Some(id) => {
                            // if issue id not in issue_id_set, skip
                            if !issue_id_set.contains(id) {
                                continue;
                            }
                            // else remove issue id from 'unfound_issue_ids'
                            else {
                                unfound_issue_ids.remove(id);
                            }
                        },
                        None => {continue;},
                    }
                }

                // Remove all issue_ids remaining in 'unfound_issue_ids' from 'issue_id_set'
                for issue_id in unfound_issue_ids.iter() {
                    issue_id_set.remove(issue_id);
                }

                // Remove all issues from 'final_issue_list' whose ids are in 'unfound_issue_ids'
                final_issue_list = final_issue_list.into_iter()
                                                    .filter_map(|e| {
                                                        match e["id"].as_str() {
                                                            Some(id) => {
                                                                if unfound_issue_ids.contains(id) {
                                                                    None
                                                                }
                                                                else {
                                                                    Some(e)
                                                                }
                                                            },
                                                            None => {None},
                                                        }
                                                    })
                                                    .collect();

            }
        }

        info!("get_issues_from_view returning final_issue_list: {:?}", final_issue_list);
        return Some(final_issue_list);
    }
    else {
        panic!("view.filters was not an Object or Null")
    }

    return None;
}

// Is it better to fetch all issues by workflow states, then filter by state_list?
// Current approach: query once for each workflow state present in state_list, then merge
// wofklowStates() -> { nodes [ { issues() } ] }
async fn get_issues_by_state( state_list: Vec<Value>, linear_config: LinearConfig ) -> Option<Vec<Value>> {
    info!("get_issues_by_state_filters received state_list: {:?}", state_list);
    // note the use of `into_iter()` to consume `items`
    let tasks: Vec<_> = state_list
    .into_iter()
    .map(|item| {
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
    .map(|item| {

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
    .map(|item| {

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