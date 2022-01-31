

use crate::linear::{
    LinearConfig,
    get_issue_due_date_category,
    client::LinearClient,
};

use crate::app::Platform;

use super::error::LinearClientError;

use std::sync::{ Arc, Mutex };


use std::collections::HashMap;


use serde_json::{ Value, Map, Number };

use crate::util::GraphQLCursor;

#[derive(Debug, PartialEq, Clone)]
pub enum ViewLoadStrategy {
    DirectQueryPaginate,
    GenericIssuePaginate,
}

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

#[derive(Debug, PartialEq, Clone)]
pub struct Filter {
    pub filter_type: FilterType,
    pub ref_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ViewLoader {
    pub load_strategy: ViewLoadStrategy,
    pub direct_filters: Vec<Filter>,

    pub filter_ignorable_groups: HashMap<String, Vec<Filter>>,


    pub direct_filter_queryable: Vec<Filter>,
    pub direct_filter_query_idx: Option<usize>,

    pub indirect_filters: Vec<Filter>,

    pub cursor: GraphQLCursor,

    pub exhausted: bool,
}

// This maps which FilterTypes can be ignored given a certain FilterType is the primary direct FilterType
lazy_static! {
    static ref IGNORABLE_FILTER_MAP: HashMap<FilterType, Vec<FilterType>> = {
        let mut m: HashMap<FilterType, Vec<FilterType>> = HashMap::new();

        // Only one Content filter per view
        m.insert(FilterType::Content, vec![]);

        m.insert(FilterType::SelectedState, vec![ FilterType::SelectedState ]);
        m.insert(FilterType::SelectedCreator, vec![ FilterType::SelectedCreator ]);
        
        m.insert(FilterType::SelectedAssignee, vec![ FilterType::SelectedAssignee, FilterType::NoAssignee ]);
        m.insert(FilterType::NoAssignee, vec![ FilterType::SelectedAssignee, FilterType::NoAssignee ]);

        m.insert(FilterType::SelectedLabel, vec![ FilterType::SelectedLabel, FilterType::NoLabel ]);
        m.insert(FilterType::NoLabel, vec![ FilterType::SelectedLabel, FilterType::NoLabel ]);

        m.insert(FilterType::SelectedProject, vec![ FilterType::SelectedProject, FilterType::NoProject ]);
        m.insert(FilterType::NoProject, vec![ FilterType::SelectedProject, FilterType::NoProject ]);

        // DueDate Filters
        m.insert(FilterType::HasDueDate, vec![FilterType::DueSoon, FilterType::DueToday, FilterType::Overdue, FilterType::NoDueDate]);
        m.insert(FilterType::DueSoon, vec![FilterType::HasDueDate, FilterType::DueToday, FilterType::Overdue, FilterType::NoDueDate]);
        m.insert(FilterType::DueToday, vec![FilterType::HasDueDate, FilterType::DueSoon, FilterType::Overdue, FilterType::NoDueDate]);
        m.insert(FilterType::Overdue, vec![FilterType::HasDueDate, FilterType::DueSoon, FilterType::DueToday, FilterType::NoDueDate]);
        m.insert(FilterType::NoDueDate, vec![FilterType::HasDueDate, FilterType::DueSoon, FilterType::DueToday, FilterType::Overdue]);

        m
    };
}



// Returns a ViewLoader created from a Linear Custom View "filters" JSON object
pub fn create_loader_from_view( filters: &Value ) -> ViewLoader {

    let load_strat: ViewLoadStrategy;

    let mut direct_filter_list: Vec<Filter> = Vec::new();


    let mut direct_filter_queryable_list: Vec<Filter> = Vec::new();
    let mut direct_filter_list_idx: Option<usize> = None;

    let mut indirect_filter_list: Vec<Filter> = Vec::new();

    // This will represent the grouped filters by FilterType,
    // within the same vec, only one FilterType needs to match any given issue
    let mut filter_type_groups: HashMap<String, Vec<Filter>> = HashMap::new();

    // init filter groups

    /*

        Content

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
    */

    filter_type_groups.insert(String::from("team"), Vec::new());
    filter_type_groups.insert(String::from("searchableContent"), Vec::new());
    filter_type_groups.insert(String::from("state"), Vec::new());
    filter_type_groups.insert(String::from("creator"), Vec::new());
    filter_type_groups.insert(String::from("label"), Vec::new());
    filter_type_groups.insert(String::from("assignee"), Vec::new());
    filter_type_groups.insert(String::from("project"), Vec::new());
    filter_type_groups.insert(String::from("priority"), Vec::new());
    filter_type_groups.insert(String::from("subscriber"), Vec::new());
    filter_type_groups.insert(String::from("dueDate"), Vec::new());


    let mut content_filter_found: bool = false;
    let mut found_content_filter: Option<Filter> = None;

    if let Value::Object(_) = filters {

        // Add 'team' filters to 'indirect_filter_list' &
        // 'filter_type_groups.get("team")'
        match &filters["team"]["id"] {
            Value::String(team_id) => {
                indirect_filter_list.push( Filter { filter_type: FilterType::SelectedTeam, ref_id: Some(team_id.to_string()) } );

                if let Some(x) = filter_type_groups.get_mut("team") {
                    x.push( Filter { filter_type: FilterType::SelectedTeam, ref_id: Some(team_id.to_string()) } );
                }
            },
            Value::Null => {
                indirect_filter_list.push( Filter { filter_type: FilterType::AllTeams, ref_id: None } );

                if let Some(x) = filter_type_groups.get_mut("team") {
                    x.push( Filter { filter_type: FilterType::AllTeams, ref_id: None } );
                }
            },
            _ => {
                error!("Custom View team id should be Value::String or Value::Null");
                panic!("Custom View team id should be Value::String or Value::Null");
            }
        }

        // Add 'state' filters to 'direct_filter_list' &
        // 'filter_type_groups.get("state")'
        if let Some(state_list) = filters["state"].as_array() {
            for state_obj in state_list.iter() {
                match &state_obj["ref"] {
                    Value::String(state_ref) => {
                        direct_filter_list.push( Filter { filter_type: FilterType::SelectedState, ref_id: Some(state_ref.to_string()) });

                        if let Some(x) = filter_type_groups.get_mut("state") {
                            x.push( Filter { filter_type: FilterType::SelectedState, ref_id: Some(state_ref.to_string()) } );
                        }
                    },
                    _ => {
                        error!("create_loader_from_view did not find 'ref' field for 'state' filter object");
                        panic!("create_loader_from_view did not find 'ref' field for 'state' filter object");
                    },
                }
            }
        };

        // ASSUMPTION: Only One Content Filter per view
        // Don't add 'searchableContent' filter to direct_filter_list (will insert in position 0 at end of method) & 
        // add to 'filter_type_groups.get("searchableContent")' & 
        // set content_filter_found = true
        if let Some(content_list) = filters["searchableContent"].as_array() {
            match &content_list[0]["ref"] {
                Value::String(content_ref) => {

                    // direct_filter_list.push( Filter { filter_type: FilterType::Content, ref_id: Some(content_ref.to_string()) });

                    if let Some(x) = filter_type_groups.get_mut("searchableContent") {
                        x.push( Filter { filter_type: FilterType::Content, ref_id: Some(content_ref.to_string()) } );
                    }

                    content_filter_found = true;
                    found_content_filter = Some(Filter { filter_type: FilterType::Content, ref_id: Some(content_ref.to_string()) });
                },
                _ => {
                    error!("create_loader_from_view did not find 'ref' field for 'searchableContent' filter object");
                    panic!("create_loader_from_view did not find 'ref' field for 'searchableContent' filter object");
                },
            }
        };

        // Add 'creator' filters to 'direct_filter_list' &
        // 'filter_type_groups.get("creator")'
        if let Some(creator_list) = filters["creator"].as_array() {
            for creator_obj in creator_list.iter() {
                match &creator_obj["ref"] {
                    Value::String(creator_ref) => {
                        direct_filter_list.push( Filter { filter_type: FilterType::SelectedCreator, ref_id: Some(creator_ref.to_string()) });

                        if let Some(x) = filter_type_groups.get_mut("creator") {
                            x.push( Filter { filter_type: FilterType::SelectedCreator, ref_id: Some(creator_ref.to_string()) } );
                        }
                    },
                    _ => {
                        error!("create_loader_from_view did not find 'ref' field for 'creator' filter object");
                        panic!("create_loader_from_view did not find 'ref' field for 'creator' filter object");
                    },
                }
            }
        };

        // Add 'assignee' filters:
        // SelectedAssignee to 'direct_filter_list' & NoAssignee to 'indirect_filter_list'
        // & all to 'filter_type_groups.get("assignee")'
        if let Some(assignee_list) = filters["assignee"].as_array() {
            for assignee_obj in assignee_list.iter() {
                match &assignee_obj["ref"] {
                    Value::String(assignee_ref) => {
                        direct_filter_list.push( Filter { filter_type: FilterType::SelectedAssignee, ref_id: Some(assignee_ref.to_string()) });

                        if let Some(x) = filter_type_groups.get_mut("assignee") {
                            x.push( Filter { filter_type: FilterType::SelectedAssignee, ref_id: Some(assignee_ref.to_string()) } );
                        }

                    },
                    // 'No Assignee' filter
                    Value::Null => {
                        indirect_filter_list.push( Filter { filter_type: FilterType::NoAssignee, ref_id: None } );

                        if let Some(x) = filter_type_groups.get_mut("assignee") {
                            x.push( Filter { filter_type: FilterType::NoAssignee, ref_id: None } );
                        }
                    },
                    _ => {
                        error!("create_loader_from_view 'assignee' filter obj must either be Value::String or Value::Null: {:?}", assignee_obj["ref"]);
                        panic!("create_loader_from_view 'assignee' filter obj must either be Value::String or Value::Null: {:?}", assignee_obj["ref"]);
                    },
                }
            }
        }

        // Add 'label' filters:
        // SelectedLabel to 'direct_filter_list' & NoLabel to 'indirect_filter_list'
        // & all to 'filter_type_groups.get("label")'
        if let Some(label_list) = filters["labels"].as_array() {
            for label_obj in label_list.iter() {
                match &label_obj["ref"] {
                    Value::String(label_ref) => {
                        direct_filter_list.push( Filter { filter_type: FilterType::SelectedLabel, ref_id: Some(label_ref.to_string()) });

                        if let Some(x) = filter_type_groups.get_mut("label") {
                            x.push( Filter { filter_type: FilterType::SelectedLabel, ref_id: Some(label_ref.to_string()) } );
                        }
                    },
                    // 'No Label' filter
                    Value::Null => {
                        indirect_filter_list.push( Filter { filter_type: FilterType::NoLabel, ref_id: None } );

                        if let Some(x) = filter_type_groups.get_mut("label") {
                            x.push( Filter { filter_type: FilterType::NoLabel, ref_id: None } );
                        }
                    },
                    _ => {
                        error!("create_loader_from_view 'labels' filter obj must either be Value::String or Value::Null: {:?}", label_obj["ref"]);
                        panic!("create_loader_from_view 'labels' filter obj must either be Value::String or Value::Null: {:?}", label_obj["ref"]);
                    },
                }
            }
        }

        // Add 'project' filters:
        // SelectedProject to 'direct_filter_list' & NoProject to 'indirect_filter_list'
        // & all to 'filter_type_groups.get("project")'

        if let Some(project_list) = filters["project"].as_array() {
            for project_obj in project_list.iter() {
                match &project_obj["ref"] {
                    Value::String(project_ref) => {
                        direct_filter_list.push( Filter { filter_type: FilterType::SelectedProject, ref_id: Some(project_ref.to_string()) });

                        if let Some(x) = filter_type_groups.get_mut("project") {
                            x.push( Filter { filter_type: FilterType::SelectedProject, ref_id: Some(project_ref.to_string()) } );
                        }

                    },
                    // 'No Project' filter
                    Value::Null => {
                        indirect_filter_list.push( Filter { filter_type: FilterType::NoProject, ref_id: None } );

                        if let Some(x) = filter_type_groups.get_mut("project") {
                            x.push( Filter { filter_type: FilterType::NoProject, ref_id: None } );
                        }
                    },
                    _ => {
                        error!("create_loader_from_view 'project' filter obj must either be Value::String or Value::Null: {:?}", project_obj["ref"]);
                        panic!("create_loader_from_view 'project' filter obj must either be Value::String or Value::Null: {:?}", project_obj["ref"]);
                    },
                }
            }
        }

        // Add 'priority' filters:
        // SelectedPriority to 'indirect_filter_list'
        // & all to 'filter_type_groups.get("priority")'

        if let Some(priority_list) = filters["priority"].as_array() {
            for priority_obj in priority_list.iter() {
                match &priority_obj["ref"] {
                    Value::Number(priority_ref) => {

                        let u64_data = priority_ref.as_u64()
                                                    .expect("Expected 'priority' 'ref' to be a Number parseable as u64")
                                                    .to_string();

                        indirect_filter_list.push( Filter { filter_type: FilterType::SelectedPriority, ref_id: Some(u64_data.clone()) });


                        if let Some(x) = filter_type_groups.get_mut("priority") {
                            x.push( Filter { filter_type: FilterType::SelectedPriority, ref_id: Some(u64_data) } );
                        }
                    },
                    _ => {
                        error!("create_loader_from_view did not find 'ref' field for 'priority' filter object: {:?}", priority_obj["ref"]);
                        panic!("create_loader_from_view did not find 'ref' field for 'priority' filter object: {:?}", priority_obj["ref"]);
                    },
                }
            }
        }

        // Add 'subscriber' filters:
        // SelectedSubscriber to 'indirect_filter_list'
        // & all to 'filter_type_groups.get("subscriber")'

        if let Some(subscriber_list) = filters["subscribers"].as_array() {
            for subscriber_obj in subscriber_list.iter() {
                match &subscriber_obj["ref"] {
                    Value::String(subscriber_ref) => {

                        indirect_filter_list.push( Filter { filter_type: FilterType::SelectedSubscriber, ref_id: Some(subscriber_ref.to_string()) });

                        if let Some(x) = filter_type_groups.get_mut("subscriber") {
                            x.push( Filter { filter_type: FilterType::SelectedSubscriber, ref_id: Some(subscriber_ref.to_string()) } );
                        }
                    },
                    _ => {
                        error!("create_loader_from_view did not find 'ref' field for 'subscribers' filter object: {:?}", subscriber_obj["ref"]);
                        panic!("create_loader_from_view did not find 'ref' field for 'subscribers' filter object: {:?}", subscriber_obj["ref"]);
                    },
                }
            }
        }

        // Add 'dueDateQualifier' filters to 'indirect_filter_list' &
        // 'filter_type_groups.get("dueDate")'
        if let Some(due_date_filter_list) = filters["dueDateQualifier"].as_array() {
            for due_date_filter in due_date_filter_list.iter() {
                match &due_date_filter["ref"] {
                    Value::String(due_date_ref) => {
                        if due_date_ref == "none" {
                            indirect_filter_list.push( Filter { filter_type: FilterType::NoDueDate, ref_id: None });

                            if let Some(x) = filter_type_groups.get_mut("dueDate") {
                                x.push( Filter { filter_type: FilterType::NoDueDate, ref_id: None } );
                            }
                        }
                        else if due_date_ref == "due" {
                            indirect_filter_list.push( Filter { filter_type: FilterType::HasDueDate, ref_id: None });

                            if let Some(x) = filter_type_groups.get_mut("dueDate") {
                                x.push( Filter { filter_type: FilterType::HasDueDate, ref_id: None } );
                            }
                        }
                        else if due_date_ref == "dueSoon" {
                            indirect_filter_list.push( Filter { filter_type: FilterType::DueSoon, ref_id: None });

                            if let Some(x) = filter_type_groups.get_mut("dueDate") {
                                x.push( Filter { filter_type: FilterType::DueSoon, ref_id: None } );
                            }
                        }
                        else if due_date_ref == "dueToday" {
                            indirect_filter_list.push( Filter { filter_type: FilterType::DueToday, ref_id: None });
                            
                            if let Some(x) = filter_type_groups.get_mut("dueDate") {
                                x.push( Filter { filter_type: FilterType::DueToday, ref_id: None } );
                            }
                        }
                        else if due_date_ref == "overdue" {
                            indirect_filter_list.push( Filter { filter_type: FilterType::Overdue, ref_id: None });

                            if let Some(x) = filter_type_groups.get_mut("dueDate") {
                                x.push( Filter { filter_type: FilterType::Overdue, ref_id: None } );
                            }
                        }
                    },
                    _ => {
                        error!("create_loader_from_view did not find 'ref' field for 'dueDateQualifier' filter object: {:?}", due_date_filter["ref"]);
                        panic!("create_loader_from_view did not find 'ref' field for 'dueDateQualifier' filter object: {:?}", due_date_filter["ref"]);
                    },
                }
            }
        };
    }


    // Set Strategy for ViewLoader: if direct_filter_list.len() > 0 { DirectQueryPaginate } else { GenericIssuePaginate }
    if !direct_filter_list.is_empty() || content_filter_found {
        load_strat = ViewLoadStrategy::DirectQueryPaginate;
    }
    else {
        load_strat = ViewLoadStrategy::GenericIssuePaginate;
    }

    // If using DirectQueryPaginate Strategy:
    //     set direct_filter_queryable_list to all Filters in direct_filter_list where
    //        direct_filter_list[x].filter_type == direct_filter_list[0].filter_type

    // If FilterType::Content is present in the direct_filter_list, only that filter should be added to direct_filter_queryable_list
    // ^^ This should be handled above by only having content filters in the direct_filter_list, if found
    if load_strat == ViewLoadStrategy::DirectQueryPaginate {
        direct_filter_list_idx = Some(0);

        // If a Content Filter is in the view, make sure that is what is queried
        if content_filter_found {
            if let Some(x) = found_content_filter {
                direct_filter_list.insert(0, x);
            }
        }

        // Pick a list of Direct Filters, all of the same type as the first Filter
        //     in direct_filter_list to be queried on when loading this view

        direct_filter_queryable_list = direct_filter_list
                                        .clone()
                                        .into_iter()
                                        .filter(|e| {
                                            e.filter_type == direct_filter_list[0].filter_type
                                        })
                                        .collect();
    }

    ViewLoader {
        load_strategy: load_strat,
    
        direct_filters: direct_filter_list,
    
        filter_ignorable_groups: filter_type_groups,
    
        direct_filter_queryable: direct_filter_queryable_list,
        direct_filter_query_idx: direct_filter_list_idx,

        indirect_filters: indirect_filter_list,

        exhausted: false,
    
        cursor: GraphQLCursor::default(),
    }
}

// Take a list of Issues and filter by all filters present in a given ViewLoader
// returns Vec of Issues matching direct_filters and indirect_filters in ViewLoader
// 'ignorable_filters' are filters which are joined to the current query results with an INTERSECT and not a UNION,
// and thus can be ignored

// Note: Ignorable Filters needs to be included in the comparison process since, e.g.
// if we have a Filter with two SelectedLabel filters and two SelectedState filters
// and we are querying on one of the SelectedLabel filters, this method will correctly ignore the non-queried SelectedLabel filter
// but it will expect both SelectedState filters to be applied simultaneously to the issue
pub fn filter_issues_by_loader( issues: Vec<Value>,
                                    team_tz_lookup: &HashMap<String,String>,
                                    tz_offset_lookup: &Arc<Mutex<HashMap<String, f64>>>,
                                    linear_config: &LinearConfig,
                                    view_loader: &ViewLoader ) -> Vec<Value> {

    // info!("filter_map_issues_by_loader - filter_ignorable_groups.get('dueDate'): {:?}", view_loader.filter_ignorable_groups.get("dueDate"));

    issues
        .into_iter()
        .filter(|e| {

            // let mut issue_is_valid = true;

            // Filter groups (one filter success validates entire group):
            /*
                filter_type_groups.insert(String::from("team"), Vec::new());
                filter_type_groups.insert(String::from("searchableContent"), Vec::new());
                filter_type_groups.insert(String::from("state"), Vec::new());
                filter_type_groups.insert(String::from("creator"), Vec::new());
                filter_type_groups.insert(String::from("label"), Vec::new());
                filter_type_groups.insert(String::from("assignee"), Vec::new());
                filter_type_groups.insert(String::from("project"), Vec::new());
                filter_type_groups.insert(String::from("priority"), Vec::new());
                filter_type_groups.insert(String::from("dueDate"), Vec::new());
            */

            let mut team_filter_met;
            // DON'T manually filter by content filter, if there is a content filter, it will be directly queried
            // let mut content_filter_met;
            let mut state_filter_met;
            let mut creator_filter_met;
            let mut label_filter_met;
            let mut assignee_filter_met;
            let mut project_filter_met;
            let mut priority_filter_met;
            let mut subscriber_filter_met;
            let mut due_date_filter_met;

            // Iterate through each list in 'filter_type_groups' and see flag whether the group is satisfied
            // at conclusion, if any group bools are false, set issue_is_valid to false

            // set filter group bools to true if no filters in group
            {
                team_filter_met = view_loader.filter_ignorable_groups.get("team")
                                        .expect("'team' key not found in filter_ignorable_groups")
                                        .is_empty();

                state_filter_met = view_loader.filter_ignorable_groups.get("state")
                                        .expect("'state' key not found in filter_ignorable_groups")
                                        .is_empty();
                
                creator_filter_met = view_loader.filter_ignorable_groups.get("creator")
                                        .expect("'creator' key not found in filter_ignorable_groups")
                                        .is_empty();

                label_filter_met = view_loader.filter_ignorable_groups.get("label")
                                        .expect("'label' key not found in filter_ignorable_groups")
                                        .is_empty();

                assignee_filter_met = view_loader.filter_ignorable_groups.get("assignee")
                                        .expect("'assignee' key not found in filter_ignorable_groups")
                                        .is_empty();

                project_filter_met = view_loader.filter_ignorable_groups.get("project")
                                        .expect("'project' key not found in filter_ignorable_groups")
                                        .is_empty();

                priority_filter_met = view_loader.filter_ignorable_groups.get("priority")
                                        .expect("'priority' key not found in filter_ignorable_groups")
                                        .is_empty();

                subscriber_filter_met = view_loader.filter_ignorable_groups.get("subscriber")
                                            .expect("'subscriber' key not found in filter_ignorable_groups")
                                            .is_empty();

                due_date_filter_met = view_loader.filter_ignorable_groups.get("dueDate")
                                        .expect("'dueDate' key not found in filter_ignorable_groups")
                                        .is_empty();
            }

            // "team"
            // Iterate through all SelectedTeam and AllTeams filters,
            // if any match e["team"]["id"] set team_filter_met = true
            for filter in view_loader.filter_ignorable_groups.get("team")
                            .expect("'team' key not found in filter_ignorable_groups")
                            .iter()
            {
                if team_filter_met { continue };
                
                match filter.filter_type {
                    FilterType::SelectedTeam => {

                        let cmp_ref_id = Value::String(filter.ref_id
                            .clone()
                            .expect("'SelectedTeam Filter must have a ref_id'")
                            .to_string());
    
                        if e["team"]["id"] == cmp_ref_id {
                            team_filter_met = true;
                        }
                    },
                    FilterType::AllTeams => {
                        team_filter_met = true;
                    },
                    _ => {
                        error!("'filter_ignorable_groups.get('team')' has invalid filter: {:?}", filter);
                        panic!("'filter_ignorable_groups.get('team')' has invalid filter: {:?}", filter);
                    }
                }
            }

            // "state"
            // Iterate through all SelectedState filters,
            // if any match e["state"]["id"] set state_filter_met = true
            for filter in view_loader.filter_ignorable_groups.get("state")
                            .expect("'state' key not found in filter_ignorable_groups")
                            .iter() 
            {
                if state_filter_met { continue };

                match filter.filter_type {
                    FilterType::SelectedState => {
                        let cmp_ref_id = Value::String(filter.ref_id
                                            .clone()
                                            .expect("'SelectedState Filter must have a ref_id'")
                                            .to_string());
                    
                        if e["state"]["id"] == cmp_ref_id {
                            state_filter_met = true;
                        }
                        
                    },
                    _ => {
                        error!("'filter_ignorable_groups.get('state')' has invalid filter: {:?}", filter);
                        panic!("'filter_ignorable_groups.get('state')' has invalid filter: {:?}", filter);
                    }
                }
            }

            // "creator"
            // Iterate through all SelectedCreator filters,
            // if any match e["creator"]["id"] set creator_filter_met = true
            for filter in view_loader.filter_ignorable_groups.get("creator")
                            .expect("'creator' key not found in filter_ignorable_groups")
                            .iter() 
            {
                if creator_filter_met { continue };

                match filter.filter_type {
                    FilterType::SelectedCreator => {
                        let cmp_ref_id = Value::String(filter.ref_id
                                            .clone()
                                            .expect("'SelectedCreator Filter must have a ref_id'")
                                            .to_string());
                    
                        if e["creator"]["id"] == cmp_ref_id {
                            creator_filter_met = true;
                        }
                    },
                    _ => { 
                        error!("'filter_ignorable_groups.get('creator')' has invalid filter: {:?}", filter);
                        panic!("'filter_ignorable_groups.get('creator')' has invalid filter: {:?}", filter);
                    }
                }
            }


            // "label"
            // Iterate through all SelectedLabel & NoLabel filters,
            // if any are found in e["labels"]["nodes"], set label_filter_met = true
            for filter in view_loader.filter_ignorable_groups.get("label")
                            .expect("'label' key not found in filter_ignorable_groups")
                            .iter()
            {
                if label_filter_met { continue };

                match filter.filter_type {
                    FilterType::SelectedLabel => {

                        let cmp_ref_id = Value::String(filter.ref_id
                            .clone()
                            .expect("'SelectedLabel Filter must have a ref_id'")
                            .to_string());

                        if let Value::Array(ref label_list) = e["labels"]["nodes"] {
                            if label_list.iter().any(|label_id| label_id["id"] == cmp_ref_id) {
                                label_filter_met = true;
                            }
                        }
                    },
                    FilterType::NoLabel => {
                        if let Value::Array(ref label_list) = e["labels"]["nodes"] {
                            if label_list.is_empty() {
                                label_filter_met = true;
                            }
                        }
                    },
                    _ => { 
                        error!("'filter_ignorable_groups.get('label')' has invalid filter: {:?}", filter);
                        panic!("'filter_ignorable_groups.get('label')' has invalid filter: {:?}", filter);
                    }
                }
            }

            // "assignee"
            // Iterate through all SelectedAssignee & NoAssignee filters,
            // if any match e["assignee"]["id"], set assignee_filter_met = true
            for filter in view_loader.filter_ignorable_groups.get("assignee")
                            .expect("'assignee' key not found in filter_ignorable_groups")
                            .iter() 
            {
                if assignee_filter_met { continue };

                match filter.filter_type {
                    FilterType::SelectedAssignee => {
                        let cmp_ref_id = Value::String(filter.ref_id
                                            .clone()
                                            .expect("'SelectedAssignee Filter must have a ref_id'")
                                            .to_string());
                    
                        if e["assignee"]["id"] == cmp_ref_id {
                            assignee_filter_met = true;
                        }
                    },
                    FilterType::NoAssignee => {
                        if Value::Null == e["assignee"] {
                            assignee_filter_met = true;
                        }
                    },
                    _ => { 
                        error!("'filter_ignorable_groups.get('assignee')' has invalid filter: {:?}", filter);
                        panic!("'filter_ignorable_groups.get('assignee')' has invalid filter: {:?}", filter);
                    }
                }
            }


            // "project"
            // Iterate through all SelectedProject & NoProject filters,
            // if any match e["project"]["id"], set project_filter_met = true
            for filter in view_loader.filter_ignorable_groups.get("project")
                            .expect("'project' key not found in filter_ignorable_groups")
                            .iter() 
            {
                if project_filter_met { continue };

                match filter.filter_type {
                    FilterType::SelectedProject => {
                        let cmp_ref_id = Value::String(filter.ref_id
                                            .clone()
                                            .expect("'SelectedProject Filter must have a ref_id'")
                                            .to_string());
                    
                        if e["project"]["id"] == cmp_ref_id {
                            project_filter_met = true;
                        }
                    },
                    FilterType::NoProject => {
                        if Value::Null == e["project"] {
                            project_filter_met = true;
                        }
                    },
                    _ => { 
                        error!("'filter_ignorable_groups.get('project')' has invalid filter: {:?}", filter);
                        panic!("'filter_ignorable_groups.get('project')' has invalid filter: {:?}", filter);
                    }
                }
            }

            // "priority"
            // Iterate through all SelectedPriority filters,
            // if any match e["priority"], set priority_filter_met = true
            for filter in view_loader.filter_ignorable_groups.get("priority")
                            .expect("'priority' key not found in filter_ignorable_groups")
                            .iter()
            {
                if priority_filter_met { continue };

                match filter.filter_type {
                    FilterType::SelectedPriority => {
                        let cmp_ref_id = Value::Number(
                                            Number::from(filter.ref_id
                                                .clone()
                                                .expect("SelectedPriority Filter must have a ref_id")
                                                .parse::<u64>()
                                                .expect("SelectedPriority Filter ref_id must be parseable as u64")
                                            )
                                        );

                        // debug!("Comparing SelectedPriority e['priority']: {:?} == cmp_ref_id: {:?}", e["priority"], cmp_ref_id);

                        if e["priority"] == cmp_ref_id {
                            debug!("Found SelectedPriority Filter Match");
                            priority_filter_met = true;
                        }
                    },
                    _ => { 
                        error!("'filter_ignorable_groups.get('priority')' has invalid filter: {:?}", filter);
                        panic!("'filter_ignorable_groups.get('priority')' has invalid filter: {:?}", filter);
                    }
                }
            }

            // "subscriber"
            // Iterate through all SelectedSubscriber filters,
            // if any are found in e["subscribers"]["nodes"], set subscriber_filter_met = true
            for filter in view_loader.filter_ignorable_groups.get("subscriber")
                            .expect("'subscriber' key not found in filter_ignorable_groups")
                            .iter()
            {
                if subscriber_filter_met { 
                    debug!("subscriber_filter_met == true, skipping");
                    continue 
                };

                match filter.filter_type {
                    FilterType::SelectedSubscriber => {
                        let cmp_ref_id = Value::String(filter.ref_id
                                            .clone()
                                            .expect("'SelectedSubscriber Filter must have a ref_id'")
                                            .to_string());


                        if let Value::Array(ref subscriber_list) = e["subscribers"]["nodes"] {
                            debug!("Comparing SelectedSubscriber cmp_ref_id: {:?} with subscriber_list: {:?}", cmp_ref_id, subscriber_list);
                            if subscriber_list.iter().any(|subscriber_id| subscriber_id["id"] == cmp_ref_id) {
                                subscriber_filter_met = true;
                            }
                        }
                    },
                    _ => { 
                        error!("'filter_ignorable_groups.get('subscriber')' has invalid filter: {:?}", filter);
                        panic!("'filter_ignorable_groups.get('subscriber')' has invalid filter: {:?}", filter);
                    }
                }
            }


            // "dueDate"
            // Iterate through all DueToday, Overdue, HasDueDate, DueSoon, NoDueDate filters,
            // get_issue_due_date() determines the current issue's correct DueDate Filter,
            // which is then compared against the current filter, if match found set due_date_filter_met = true
            for filter in view_loader.filter_ignorable_groups.get("dueDate")
                            .expect("'dueDate' key not found in filter_ignorable_groups")
                            .iter() 
            {
                if due_date_filter_met { continue };

                debug!("filter_map_issues_by_loader: found dueDate filters to filter issues by");

                let tz_offset_lookup_lock = tz_offset_lookup.lock().unwrap();

                let issue_due_date_filter = get_issue_due_date_category(    &team_tz_lookup,
                                                &tz_offset_lookup_lock,
                                                e["team"]["id"].clone(),
                                                e["dueDate"].clone(),
                                                &linear_config
                                            );
                debug!("filter_map_issues_by_loader: issue_due_date_filter: {:?}", issue_due_date_filter);

                match filter.filter_type {

                    FilterType::DueToday => {
                        if issue_due_date_filter == FilterType::DueToday {
                            due_date_filter_met = true;
                        }
                    },
                    FilterType::Overdue => {
                        if issue_due_date_filter == FilterType::Overdue {
                            due_date_filter_met = true;
                        }
                    },
                    FilterType::HasDueDate => {
                        if issue_due_date_filter == FilterType::DueToday
                            || issue_due_date_filter == FilterType::DueSoon
                            || issue_due_date_filter == FilterType::Overdue
                            || issue_due_date_filter == FilterType::HasDueDate
                        {
                            due_date_filter_met = true;
                        }
                    },
                    FilterType::DueSoon => {
                        if issue_due_date_filter == FilterType::DueToday
                            || issue_due_date_filter == FilterType::DueSoon
                        {
                            due_date_filter_met = true;
                        }
                    },
                    FilterType::NoDueDate => {
                        if issue_due_date_filter == FilterType::NoDueDate
                        {
                            due_date_filter_met = true;
                        }
                    },
                    _ => {
                        error!("'filter_ignorable_groups.get('dueDate')' has invalid filter: {:?}", filter);
                        panic!("'filter_ignorable_groups.get('dueDate')' has invalid filter: {:?}", filter);
                    }
                }
            }


            if  !team_filter_met ||
                !state_filter_met ||
                !creator_filter_met ||
                !label_filter_met ||
                !assignee_filter_met ||
                !project_filter_met ||
                !priority_filter_met ||
                !subscriber_filter_met ||
                !due_date_filter_met
            {
                debug!("team_filter_met: {:?} state_filter_met: {:?} creator_filter_met: {:?} label_filter_met: {:?} assignee_filter_met: {:?} project_filter_met: {:?} due_date_filter_met: {:?}",
                            team_filter_met,
                            state_filter_met,
                            creator_filter_met,
                            label_filter_met,
                            assignee_filter_met,
                            project_filter_met,
                            due_date_filter_met);
                false
            }
            else {
                true
            }
        })
        .collect()
}

pub fn deduplicate_issue_list ( issues_to_filter: Vec<Value>, found_issue_list: &mut Vec<Value>, dedup_list: &mut Vec<Value> ) -> Vec<Value> {
    issues_to_filter
        .into_iter()
        .filter(|e| {
            if found_issue_list.is_empty() && dedup_list.is_empty() {
                return true;
            }
            // Check both found_issue_list and dedup_list for duplicates
            match found_issue_list.iter().any(|x| {
                x["id"] == e["id"]
            }) {
                true => { return false },
                false => {  }
            };
            !dedup_list.iter().any(|x| { x["id"] == e["id"] })
        })
        .collect()
}

pub async fn generic_issue_fetch (  view_loader: &mut ViewLoader,
                                    dedup_list: &mut Vec<Value>,
                                    request_num: &mut u32,
                                    team_tz_lookup: &HashMap<String,String>,
                                    tz_offset_lookup: &Arc<Mutex<HashMap<String, f64>>>,
                                    linear_config: &LinearConfig,
                                ) -> Vec<Value> {

    // Determine if a SelectedTeam filter is present in view_loader.filter_ignorable_groups.get('team')
    // if so: query using fetch_issues_by_team
    // else: query using fetch_all_issues

    let fetch_by_team: bool = view_loader.filter_ignorable_groups.get("team")
                                    .expect("'team' key not found in filter_ignorable_groups")
                                    .iter()
                                    .any(|e| e.filter_type == FilterType::SelectedTeam);
    
    let mut selected_team_id: String = String::default();

    if fetch_by_team {
        let selected_team_idx = view_loader.filter_ignorable_groups.get("team")
                                    .expect("'team' key not found in filter_ignorable_groups")
                                    .iter()
                                    .position(|e| e.filter_type == FilterType::SelectedTeam)
                                    .expect("'fetch_by_team is true, but no FilterType::SelectedTeam Filter found in filter_ignorable_groups.get('team')'");

        selected_team_id = view_loader.filter_ignorable_groups.get("team")
                                .expect("'team' key not found in filter_ignorable_groups")
                                [selected_team_idx]
                                .ref_id
                                .clone()
                                .expect("'SelectedTeam Filter must have a ref_id'");
    }

    let mut found_issue_list: Vec<Value> = Vec::new();

    let mut loop_num: u16 = 0;

    loop {

        // If Indirect Query is exhausted
        if view_loader.cursor.platform == Platform::Linear && !view_loader.cursor.has_next_page {
            // No more Pages in Indirect Query remaining, return found_issues_list
            view_loader.exhausted = true; 
            debug!("Indirect Query - no more issues to query, returning found_issues_list");
            return found_issue_list;
        }

        let query_result: Result<Value, LinearClientError>;

        match fetch_by_team {
            true => {
                let mut variables: Map<String, Value> = Map::new();

                variables.insert(String::from("ref"), Value::String(selected_team_id.clone()));

                query_result = LinearClient::get_issues_by_team(linear_config.clone(), Some(view_loader.cursor.clone()), variables, true).await;
            },
            false => {
                query_result = LinearClient::get_all_issues(linear_config.clone(), Some(view_loader.cursor.clone()), true).await;
            }
        }

        if let Ok(response) = query_result {

            // Increment request_num here
            *request_num += 1;

            debug!("Current Indirect Filter Query Response: {:?}", response);

            // Filter returned Issues by all other loader filters
            // and add remainder to final_issue_list

            let mut issues_to_filter: Vec<Value>;
            
            match response["issue_nodes"].as_array() {
                Some(resp_issue_data) => {
                    issues_to_filter = resp_issue_data.clone();
                },
                None => {
                    error!("'issue_nodes' invalid format: {:?}", response["issue_nodes"]);
                    panic!("'issue_nodes' invalid format");
                }
            }

            debug!("issues_to_filter.len(): {:?}", issues_to_filter.len());

            // Remove any Issues from issues_to_filter that are already in found_issue_list

            issues_to_filter = deduplicate_issue_list(issues_to_filter, &mut found_issue_list, dedup_list);

            debug!("issues_to_filter.len() (dedup): {:?}", issues_to_filter.len());

            // Filter queried Issues by 
            let mut filtered_issue_list: Vec<Value> = filter_issues_by_loader(
                                            issues_to_filter,
                                            &team_tz_lookup,
                                            &tz_offset_lookup,
                                            &linear_config,
                                            &view_loader
                                        );
            
            debug!("filtered_issue_list.len(): {:?}", filtered_issue_list.len());


            if !filtered_issue_list.is_empty() {
                found_issue_list.append(&mut filtered_issue_list);
            }


            // Update GraphQLCursor
            match GraphQLCursor::linear_cursor_from_page_info(response["cursor_info"].clone()) {
                Some(new_cursor) => {
                    view_loader.cursor = new_cursor;
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

pub async fn direct_issue_fetch (   view_loader: &mut ViewLoader,
                                    dedup_list: &mut Vec<Value>,
                                    request_num: &mut u32,
                                    team_tz_lookup: &HashMap<String,String>,
                                    tz_offset_lookup: &Arc<Mutex<HashMap<String, f64>>>,
                                    linear_config: &LinearConfig
                                ) -> Vec<Value> {

    let mut query_list_idx: usize;

    // Assign to query_list_idx if view_loader has a direct_filter_query_idx
    // if not, then this is not a DirectQueryPaginate strategy, return
    if let Some(x) = view_loader.direct_filter_query_idx {
        query_list_idx = x;
    }
    else {
        error!("'direct_issue_fetch' - view_loader.direct_filter_query_idx must be Some()");
        panic!("'direct_issue_fetch' - view_loader.direct_filter_query_idx must be Some()");
    }

    debug!("Direct Filter List: {:?}", view_loader.direct_filter_queryable);

    let mut found_issue_list: Vec<Value> = Vec::new();

    let mut loop_num: u16 = 0;

    // Continue querying until full page of issues loaded or no more issues to scan
    loop {

        // If cursor.platform == Platform::Linear && cursor.hasNextpage == false
        //     If (query_list_idx+1) < view_loader.direct_filter_queryable.len():
        //         increment query_list_idx (update view_loader.direct_filter_query_idx as well)
        //         set view_loader.cursor = GraphQLCursor::default()
        //     else:
        //        

        // If current Direct Query is exhausted
        if view_loader.cursor.platform == Platform::Linear && !view_loader.cursor.has_next_page {
            // If more Direct Queries remaining, increment index and reset cursor
            if (query_list_idx+1) < view_loader.direct_filter_queryable.len() {

                debug!("Current Direct Query exhausted, shifting to next Direct Query");

                query_list_idx += 1;
                view_loader.direct_filter_query_idx = Some(query_list_idx);
                view_loader.cursor = GraphQLCursor::default();
            }
            // No more Direct Queries remaining, return found_issues_list
            else {
                view_loader.exhausted = true; 
                debug!("No more Direct Queries remaining, returning found_issues_list");
                return found_issue_list;
            }
        }

        let current_direct_filter: &Filter = &view_loader.direct_filter_queryable[query_list_idx];

        debug!("Current Direct Filter : {:?}", current_direct_filter);

        let query_result: Result<Value, LinearClientError>;

        // Fetch Issues from the current Direct Filter using the current cursor
        match &current_direct_filter.filter_type {
            FilterType::Content => {
                if let Some(ref_id) = &current_direct_filter.ref_id {
                    let mut variables: Map<String, Value> = Map::new();
                    variables.insert(String::from("ref"), Value::String(ref_id.clone()));

                    query_result = LinearClient::get_issues_by_content(linear_config.clone(), Some(view_loader.cursor.clone()), variables, true).await;

                }
                else {
                    error!("Content Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                    panic!("Content Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                }
            },
            FilterType::SelectedState => {
                if let Some(ref_id) = &current_direct_filter.ref_id {
                    let mut variables: Map<String, Value> = Map::new();
                    variables.insert(String::from("ref"), Value::String(ref_id.clone()));

                    query_result = LinearClient::get_issues_by_workflow_state(linear_config.clone(), Some(view_loader.cursor.clone()), variables, true).await;

                }
                else {
                    error!("SelectedState Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                    panic!("SelectedState Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                }
            },
            FilterType::SelectedCreator => {
                if let Some(ref_id) = &current_direct_filter.ref_id {
                    let mut variables: Map<String, Value> = Map::new();
                    variables.insert(String::from("ref"), Value::String(ref_id.clone()));

                    query_result = LinearClient::get_issues_by_creator(linear_config.clone(), Some(view_loader.cursor.clone()), variables, true).await;

                }
                else {
                    error!("SelectedCreator Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                    panic!("SelectedCreator Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                }
            },
            FilterType::SelectedAssignee => {
                if let Some(ref_id) = &current_direct_filter.ref_id {
                    let mut variables: Map<String, Value> = Map::new();
                    variables.insert(String::from("ref"), Value::String(ref_id.clone()));

                    query_result = LinearClient::get_issues_by_assignee(linear_config.clone(), Some(view_loader.cursor.clone()), variables, true).await;

                }
                else {
                    error!("SelectedCreator Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                    panic!("SelectedCreator Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                }
            },
            FilterType::SelectedLabel => {
                if let Some(ref_id) = &current_direct_filter.ref_id {
                    let mut variables: Map<String, Value> = Map::new();
                    variables.insert(String::from("ref"), Value::String(ref_id.clone()));

                    query_result = LinearClient::get_issues_by_label(linear_config.clone(), Some(view_loader.cursor.clone()), variables, true).await;

                }
                else {
                    error!("SelectedCreator Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                    panic!("SelectedCreator Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                }
            },
            FilterType::SelectedProject => {
                if let Some(ref_id) = &current_direct_filter.ref_id {
                    let mut variables: Map<String, Value> = Map::new();
                    variables.insert(String::from("ref"), Value::String(ref_id.clone()));

                    query_result = LinearClient::get_issues_by_project(linear_config.clone(), Some(view_loader.cursor.clone()), variables, true).await;

                }
                else {
                    error!("SelectedProject Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                    panic!("SelectedProject Filter cannot have 'None' for 'ref_id' - Filter: {:?}", current_direct_filter);
                }
            }

            _ => {
                error!("Invalid Label found in view_loader.direct_filter_queryable");
                panic!("Invalid Label found in view_loader.direct_filter_queryable");
            }
        }


        if let Ok(response) = query_result {

            // Increment request_num here
            *request_num += 1;

            debug!("Current Direct Filter Query Response: {:?}", response);

            // Filter returned Issues by all other loader filters
            // and add remainder to final_issue_list

            let mut issues_to_filter: Vec<Value>;
            
            match response["issue_nodes"].as_array() {
                Some(resp_issue_data) => {
                    issues_to_filter = resp_issue_data.clone();
                },
                None => {
                    error!("'issue_nodes' invalid format: {:?}", response["issue_nodes"]);
                    panic!("'issue_nodes' invalid format");
                }
            }

            debug!("issues_to_filter.len(): {:?}", issues_to_filter.len());

            // Remove any Issues from issues_to_filter that are already in found_issue_list

            issues_to_filter = deduplicate_issue_list(issues_to_filter, &mut found_issue_list, dedup_list);

            debug!("issues_to_filter.len() (dedup): {:?}", issues_to_filter.len());

            // Filter queried Issues by 
            let mut filtered_issue_list: Vec<Value> = filter_issues_by_loader(
                                            issues_to_filter,
                                            &team_tz_lookup,
                                            &tz_offset_lookup,
                                            &linear_config,
                                            &view_loader
                                        );

            debug!("filtered_issue_list.len(): {:?}", filtered_issue_list.len());

            
            if !filtered_issue_list.is_empty() {
                found_issue_list.append(&mut filtered_issue_list);
            }


            // Update GraphQLCursor
            match GraphQLCursor::linear_cursor_from_page_info(response["cursor_info"].clone()) {
                Some(new_cursor) => {
                    view_loader.cursor = new_cursor;
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

pub async fn optimized_view_issue_fetch (   view_obj: &Value,
                                            view_loader_option: Option<ViewLoader>,
                                            team_tz_lookup: HashMap<String,String>,
                                            tz_offset_lookup: Arc<Mutex<HashMap<String, f64>>>,
                                            issue_data: Arc<Mutex<Vec<Value>>>,
                                            linear_config: LinearConfig
                                        ) -> ( Vec<Value>, ViewLoader, u32 ) {

    info!("View Resolver received view_obj: {:?}", view_obj);

    let filters = view_obj["filters"].clone();

    let mut view_loader =  if let Some(loader) = view_loader_option { loader } else { create_loader_from_view(&filters) };

    debug!("ViewLoader: {:?}", view_loader);

    let mut dedup_list: Vec<Value> = Vec::new();

    // Append currently found issues from 'issue_data' to 'dedup_list'
    {
        let issue_data_lock = issue_data.lock().unwrap();
        dedup_list.append(&mut issue_data_lock.clone());
    }




    let mut request_num: u32 = 0;
    let mut found_issue_list: Vec<Value> = Vec::new();

    if view_loader.load_strategy == ViewLoadStrategy::DirectQueryPaginate {
        found_issue_list = direct_issue_fetch(  &mut view_loader, &mut dedup_list,
                                                &mut request_num, &team_tz_lookup,
                                                &tz_offset_lookup, &linear_config).await;
    }
    else if view_loader.load_strategy == ViewLoadStrategy::GenericIssuePaginate {
        found_issue_list = generic_issue_fetch( &mut view_loader, &mut dedup_list,
                                                &mut request_num, &team_tz_lookup,
                                                &tz_offset_lookup, &linear_config).await;
    }

    info!("'optimized_view_issue_fetch' returning found_issue_list.len(): {:?}", found_issue_list.len());
    info!("'optimized_view_issue_fetch' returning found_issue_list: {:?}", found_issue_list);

    return (found_issue_list, view_loader, request_num);
}