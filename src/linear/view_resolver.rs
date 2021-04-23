
use serde_json::Value;
use crate::linear::LinearConfig;
use crate::linear::client::LinearClient;

use crate::errors::ConfigError;
use crate::errors::GraphQLRequestError;
use super::error::LinearClientError;

use std::sync::{ Arc, Mutex };

use std::collections::HashSet;

use crate::util::GraphQLCursor;

#[derive(Debug, PartialEq)]
pub enum ViewLoadStrategy {
    DirectQueryPaginate,
    GenericIssuePaginate,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterType {
    SelectedState,
    SelectedCreator,
    SelectedAssignee,
    SelectedLabel,

    NoLabel,
    NoAssignee,

}

#[derive(Debug, PartialEq, Clone)]
pub struct Filter {
    pub filter_type: FilterType,
    pub ref_id: Option<String>,    
}

#[derive(Debug)]
pub struct ViewLoader {
    pub load_strategy: ViewLoadStrategy,
    
    pub direct_filters: Vec<Filter>,


    pub direct_filter_queryable: Option<Vec<Filter>>,
    pub direct_filter_query_idx: Option<usize>,

    pub indirect_filters: Vec<Filter>,

    pub cursor: Option<GraphQLCursor>,
}

pub fn create_loader_from_view( filters: &Value ) -> ViewLoader {

    let mut load_strat: ViewLoadStrategy;

    let mut direct_filter_list: Vec<Filter> = Vec::new();
    let mut direct_filter_queryable_list: Option<Vec<Filter>> = None;
    let mut direct_filter_list_idx: Option<usize> = None;

    let mut indirect_filter_list: Vec<Filter> = Vec::new();


    // Add 'state' filters
    if let Value::Object(_) = filters {
        match filters["state"].as_array() {
            Some(state_list) => {
                for state_obj in state_list.iter() {
                    match &state_obj["ref"] {
                        Value::String(state_ref) => {
                            direct_filter_list.push( Filter { filter_type: FilterType::SelectedState, ref_id: Some(state_ref.to_string()) });
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        };
    }

    // Add 'creator' filters
    if let Value::Object(_) = filters {
        match filters["creator"].as_array() {
            Some(creator_list) => {
                for creator_obj in creator_list.iter() {
                    match &creator_obj["ref"] {
                        Value::String(creator_ref) => {
                            direct_filter_list.push( Filter { filter_type: FilterType::SelectedCreator, ref_id: Some(creator_ref.to_string()) });
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        };
    }

    // Add 'assignee' filters, including 'No Assignee'
    if let Value::Object(_) = filters {
        match filters["assignee"].as_array() {
            Some(assignee_list) => {
                for assignee_obj in assignee_list.iter() {
                    match &assignee_obj["ref"] {
                        Value::String(assignee_ref) => {
                            direct_filter_list.push( Filter { filter_type: FilterType::SelectedAssignee, ref_id: Some(assignee_ref.to_string()) });
                        },
                        // 'No Assignee' filter
                        Value::Null => {
                            indirect_filter_list.push( Filter { filter_type: FilterType::NoAssignee, ref_id: None } );
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        };
    }

    // Add 'label' filters, including 'No Label'
    if let Value::Object(_) = filters {
        match filters["labels"].as_array() {
            Some(label_list) => {
                for label_obj in label_list.iter() {
                    match &label_obj["ref"] {
                        Value::String(label_ref) => {
                            direct_filter_list.push( Filter { filter_type: FilterType::SelectedLabel, ref_id: Some(label_ref.to_string()) });
                        },
                        // 'No Label' filter
                        Value::Null => {
                            indirect_filter_list.push( Filter { filter_type: FilterType::NoLabel, ref_id: None } );
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        };
    }

    // Set Strategy for ViewLoader: if direct_filter_list.len() > 0 { DirectQueryPaginate } else { GenericIssuePaginate }
    if direct_filter_list.len() > 0 { 
        load_strat = ViewLoadStrategy::DirectQueryPaginate;
    }
    else {
        load_strat = ViewLoadStrategy::GenericIssuePaginate;
    }

    // If using DirectQueryPaginate Strategy:
    //     set direct_filter_queryable_list to all Filters in direct_filter_list where
    //        direct_filter_list[x].filter_type == direct_filter_list[0].filter_type
    if load_strat == ViewLoadStrategy::DirectQueryPaginate {
        direct_filter_list_idx = Some(0);
        direct_filter_queryable_list = Some(direct_filter_list
                                        .clone()
                                        .into_iter()
                                        .filter_map(|e| {
                                            if e.filter_type == direct_filter_list[0].filter_type {
                                                Some(e) 
                                            }
                                            else {
                                                None
                                            }
                                        })
                                        .collect()
                                    );
    }

    ViewLoader {
        load_strategy: load_strat,
    
        direct_filters: direct_filter_list,
    
    
        direct_filter_queryable: direct_filter_queryable_list,
        direct_filter_query_idx: direct_filter_list_idx,
    
        indirect_filters: indirect_filter_list,
    
        cursor: None,
    }
}


// Accepts a custom view object: &Value
// TODO: Certain filter resolvers may have to paginate in order to create accurate view and filter lists
pub async fn get_issues_from_view( view_obj: &Value, linear_config: LinearConfig ) -> Option<Vec<Value>> {
    // Determine which filter resolver methods to call based on available attributes,
    // and perform an Intersect operation on returned issues to determine 

    info!("View Resolver received view_obj: {:?}", view_obj);

    let filters = view_obj["filters"].clone();

    info!("ViewLoader: {:?}", create_loader_from_view(&filters));

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
            },
            _ => {},
        }

        // Handle 'labels' filters
        match filters["labels"].as_array() {
            Some(label_list) => {
                filter_component_results.push(
                    get_issues_by_label(label_list.clone(), linear_config.clone()).await
                );
            },
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
                                        // e: Option<Vec<Value>>
                                        match e {
                                            Some(_) => e,
                                            None => None,
                                        }
                                    })
                                    .collect();
        debug!("resolver_issue_values: {:?}", resolver_issue_values);
    
        for (idx, value_list) in resolver_issue_values.iter().enumerate() {

            // If first list, add all issue ids to issue_id_set 
            //     for every issue id not already in issue_id_set, add issue to final issue list

            if idx == 0 {
                let mut current_issue_id: String;
                for issue_obj in value_list.iter() {
                    let mut not_already_in_set = false;

                    match issue_obj["id"].as_str() {
                        Some(id) => {
                            debug!("Inserting Issue id ${:?} into issue_id_set", id);
                            not_already_in_set = issue_id_set.insert(String::from(id));
                        },
                        None => {
                            debug!("Skipping, no 'id' found - issue_obj: {:?}", issue_obj);
                            continue;
                        },
                    }
                    // Issue id not already in issue_id_set, add issue to final issue list
                    if not_already_in_set == true {
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
            debug!("Iter {:?}, issue_id_set: {:?}", idx, issue_id_set);
            debug!("Iter {:?}, final_issue_list: {:?}", idx, final_issue_list);
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
    /*
    // verify that we've got the results
    for item in &items {
        info!("get_issues_by_workflow_state Result: {:?}", item);
    }
    */

    let issues: Vec<Value> = items
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    })
                    .map(|e| match e {
                        Value::Array(x) => {x},
                        _ => {vec![]}
                    })
                    .flatten()
                    .collect();
    debug!("get_issues_by_workflow_state Issues: {:?}", issues);


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
    /*
    // verify that we've got the results
    for item in &items {
        info!("get_issues_by_assignee Result: {:?}", item);
    }
    */

    let issues: Vec<Value> = items
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    })
                    .map(|e| match e {
                        Value::Array(x) => {x},
                        _ => {vec![]}
                    })
                    .flatten()
                    .collect();
    debug!("get_issues_by_assignee Issues: {:?}", issues);


    return Some(issues);

}

// Note: Currently will ignore 'No Label' filter
async fn get_issues_by_label ( label_list: Vec<Value>, linear_config: LinearConfig ) -> Option<Vec<Value>> {
    info!("get_issues_by_label received label_list: {:?}", label_list);

    // note the use of `into_iter()` to consume `items`
    let tasks: Vec<_> = label_list
        .into_iter()
        .map(|item| {

            let mut invalid_filter = false;

            // If 'item' does not have a ref, is 'No Label' filter, skip
            if let Value::Null = item["ref"] {
                invalid_filter = true;
            }

            info!("Spawning Get Issue By Label Task");
            let temp_config = linear_config.clone();
            tokio::spawn(async move {
                if invalid_filter == true {
                    return Err(LinearClientError::InvalidConfig(
                            ConfigError::InvalidParameter { parameter: String::from("View Label filter") }
                        )
                    );
                };
                match item.as_object() {
                    Some(label_obj) => {
                        let label_issues = LinearClient::get_issues_by_label( temp_config, label_obj.clone() ).await;
                        label_issues
                    },
                    _ => {
                        Err( LinearClientError::InvalidConfig( ConfigError::InvalidParameter { parameter: String::from("Label Obj not found") } ) )
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

    let issues: Vec<Value> = items
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    })
                    .map(|e| match e {
                        Value::Array(x) => {x},
                        _ => {vec![]}
                    })
                    .flatten()
                    .collect();
    debug!("get_issues_by_label Issues: {:?}", issues);


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
    /*
    // verify that we've got the results
    for item in &items {
        info!("get_issues_by_creator Result: {:?}", item);
    }
    */

    let issues: Vec<Value> = items
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    })
                    .map(|e| match e {
                        Value::Array(x) => {x},
                        _ => {vec![]}
                    })
                    .flatten()
                    .collect();
    debug!("get_issues_by_creator Issues: {:?}", issues);


    return Some(issues);
}