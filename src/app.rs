use crate::util;
use crate::linear;
use crate::components;
use crate::network;

use network::IOEvent as IOEvent;

use util::StatefulList as StatefulList;
use util::GraphQLCursor;

use tokio::sync::oneshot;

use std::sync::{Arc, Mutex};

use crate::linear::LinearConfig;

use serde_json::Value;

use tui::{
    widgets::{ TableState },
};

pub struct ViewLoadBundle {
    pub linear_config: LinearConfig,
    pub item_filter: Value,
    pub table_data: Arc<Mutex<Option<Value>>>,
    pub tx: tokio::sync::mpsc::Sender<IOEvent>,
}


pub enum Route {
    ActionSelect,
    DashboardViewDisplay,
    CustomViewSelect,
    TeamSelect,
    LinearInterface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    Na,
    Linear,
    Github,
}
// linear_team_select

// App holds the state of the application
pub struct App<'a> {
    // current route
    pub route: Route,
    /// Current value of the Command string
    pub cmd_str: String,
    // LinearClient
    linear_client: linear::client::LinearClient,

    // Linear Custom View Select
    pub linear_custom_view_select: components::linear_custom_view_select::LinearCustomViewSelect,
    // Selected Custom View
    pub linear_selected_custom_view_idx: Option<usize>,
    // Linear Custom View Cursor
    pub linear_custom_view_cursor: Arc<Mutex<GraphQLCursor>>,

    // Linear Dashboard Custom View List Display
    pub dashboard_view_display: components::dashboard_view_display::DashboardViewDisplay,
    // Linear Dashbaord Custom View List
    pub linear_dashboard_view_list: Vec<Option<serde_json::Value>>,
    pub linear_dashboard_view_idx: Option<usize>,

    // Linear Dashboard View Panel Display
    
    // Linear Dashboard 'DashboardViewPanel' components
    pub linear_dashboard_view_panel_list: Arc<Mutex<Vec<components::dashboard_view_panel::DashboardViewPanel>>>,
    pub linear_dashboard_view_panel_selected: Option<usize>,
    pub view_panel_issue_selected: Option<TableState>,


    // Linear Team Select State
    pub linear_team_select: components::linear_team_select::LinearTeamSelectState,
    // Selected Linear Team
    pub linear_selected_team_idx: Option<usize>,

    // Linear Issue Display State
    pub linear_issue_display: components::linear_issue_display::LinearIssueDisplay,
    // Selected Linear Issue
    pub linear_selected_issue_idx: Option<usize>,
    // Linear Issue Display Cursor
    pub linear_issue_cursor: Arc<Mutex<GraphQLCursor>>,

    // Linear Workflow Select State
    pub linear_workflow_select: components::linear_workflow_state_display::LinearWorkflowStateDisplayState,
    // Selected Linear Workflow State
    pub linear_selected_workflow_state_idx: Option<usize>,
    // Draw Workflow State Selection panel
    pub  linear_draw_workflow_state_select: bool,


    // Available actions
    pub actions: StatefulList<&'a str>,
}



impl<'a> Default for App<'a> {
    fn default() -> App<'a> {
        App {
            route: Route::ActionSelect,
            cmd_str: String::new(),

            linear_client: linear::client::LinearClient::default(),

            linear_custom_view_select: components::linear_custom_view_select::LinearCustomViewSelect::default(),
            linear_selected_custom_view_idx: None,
            linear_custom_view_cursor: Arc::new(Mutex::new(GraphQLCursor::default())),

            dashboard_view_display: components::dashboard_view_display::DashboardViewDisplay::default(),
            linear_dashboard_view_list: vec![ None, None, None, None, None, None ],
            linear_dashboard_view_idx: None,

            linear_dashboard_view_panel_list: Arc::new(Mutex::new(Vec::new())),
            linear_dashboard_view_panel_selected: None,
            view_panel_issue_selected: None,

            linear_team_select: components::linear_team_select::LinearTeamSelectState::default(),
            // Null
            linear_selected_team_idx: None,
 
            linear_issue_display: components::linear_issue_display::LinearIssueDisplay::default(),
            linear_selected_issue_idx: None,
            linear_issue_cursor: Arc::new(Mutex::new(util::GraphQLCursor::platform_cursor(Platform::Linear))),

            
            linear_workflow_select: components::linear_workflow_state_display::LinearWorkflowStateDisplayState::default(),
            linear_selected_workflow_state_idx: None,

            linear_draw_workflow_state_select: false,

            actions: util::StatefulList::with_items(vec![
                "Modify Dashboard",
                "Create New Custom View",
            ]),
        }
    }
}







impl<'a> App<'a> {


    pub fn draw_issue_state_select(&self, platform: Platform) -> &bool {
        match platform {
            Platform::Linear => { return &self.linear_draw_workflow_state_select },
            Platform::Github => { return &false },
            Platform::Na => { return &false }
        }
    }

    pub fn set_draw_issue_state_select(&mut self, platform: Platform, v: bool) {
        match platform {
            Platform::Linear => { self.linear_draw_workflow_state_select = v },
            Platform::Github => { },
            Platform::Na => { },
        };
    }

    pub fn change_route(&mut self, route: Route, tx: &tokio::sync::mpsc::Sender<IOEvent>) {
        match route {

            // Create DashboardViewPanel components for each Some in app.linear_dashboard_view_list
            // and set app.linear_dashboard_view_panel_list
            // Load all Dashboard Views
            Route::ActionSelect => {
                self.dispatch_event("load_dashboard_views", &tx);
            },

            Route::DashboardViewDisplay => {},
            Route::CustomViewSelect => {
                // TODO: Clear any previous CustomViewSelect related values on self
                self.linear_custom_view_select = components::linear_custom_view_select::LinearCustomViewSelect::default();
                self.linear_selected_custom_view_idx = None;
                self.linear_custom_view_cursor = Arc::new(Mutex::new(GraphQLCursor::default()));

                self.dispatch_event("load_custom_views", tx);

            },
            Route::TeamSelect => {

                let tx2 = tx.clone();

                let api_key = self.linear_client.config.api_key.clone();

                let team_data_handle = self.linear_team_select.teams_data.clone();


                let t1 = tokio::spawn(async move {

                    let (resp_tx, resp_rx) = oneshot::channel();

                    let cmd = IOEvent::LoadLinearTeams { api_key: api_key, resp: resp_tx };
                    tx2.send(cmd).await.unwrap();

                    let res = resp_rx.await.ok();

                    info!("LoadLinearTeams IOEvent returned: {:?}", res);

                    let mut team_data_lock = team_data_handle.lock().unwrap();

                    match res {
                        Some(x) => {
                            *team_data_lock = x;
                        }
                        None => {},
                    }

                    info!("New self.linear_team_select.teams_data: {:?}", team_data_lock);
                });


                // self.linear_team_select.load_teams(&mut self.linear_client).await;
            },

            Route::LinearInterface => {

                let tx3 = tx.clone();
                
                // let api_key = self.linear_client.config.api_key.clone();
                let linear_config = self.linear_client.config.clone();

                let team_data_handle = self.linear_team_select.teams_data.clone();
                let team_issue_handle = self.linear_issue_display.issue_table_data.clone();

                let team_issue_cursor_handle = self.linear_issue_cursor.clone();

                match self.linear_selected_team_idx {
                    Some(idx) => {
                        // Arc<Option<T>> -> Option<&T>
                        match &*team_data_handle.lock().unwrap() {
                            Some(data) => {
                                
                                let team = data[idx].clone();

                                match team {
                                    serde_json::Value::Null => return,
                                    _ => {},
                                }

                                let t2 = tokio::spawn(async move {
                    
                                    let (resp2_tx, resp2_rx) = oneshot::channel();
                
                                    let cmd = IOEvent::LoadLinearIssues { linear_config: linear_config, selected_team: team, resp: resp2_tx };
                                    tx3.send(cmd).await.unwrap();
                
                                    let res = resp2_rx.await.ok();

                                    info!("LoadLinearIssues IOEvent returned: {:?}", res);

                                    let mut issue_data_lock = team_issue_handle.lock().unwrap();
                                    let mut issue_cursor_data_lock = team_issue_cursor_handle.lock().unwrap();

                                    match res {
                                        Some(x) => {
                                            match x {
                                                Some(y) => {
                                                    match y["issues"] {
                                                        serde_json::Value::Array(_) => {
                                                            *issue_data_lock = Some(y["issues"].clone());
                                                        },
                                                        _ => {},
                                                    };
                                                    match GraphQLCursor::linear_cursor_from_page_info(y["cursor_info"].clone()) {
                                                        Some(z) => {
                                                            info!("Updating issue_cursor_data_lock to: {:?}", z);
                                                            *issue_cursor_data_lock = z;
                                                        },
                                                        None => {},
                                                    }
                                                },
                                                None => {

                                                }
                                            };
                                        }
                                        None => {},
                                    }
                
                                    // info!("New self.linear_team_select.teams_data: {:?}", issue_data_lock);
                
                
                                });
                            }
                            None => {},
                        }
                    },
                    _ => {return;},
                }
            },
        }
        self.route = route;
    }

    pub fn dispatch_event(&mut self, event_name: &str, tx: &tokio::sync::mpsc::Sender<IOEvent>) {

        match event_name {

            "load_custom_views" => {
                // TODO: Clear any previous CustomViewSelect related values on self

                let tx2 = tx.clone();

                let linear_config = self.linear_client.config.clone();

                let view_data_handle = self.linear_custom_view_select.view_table_data.clone();

                let view_cursor_handle = self.linear_custom_view_cursor.clone();

                let view_cursor_handle = self.linear_custom_view_cursor.lock().unwrap();
                let view_cursor: GraphQLCursor = view_cursor_handle.clone();
                drop(view_cursor_handle);

                let view_cursor_handle = self.linear_custom_view_cursor.clone();



                let t1 = tokio::spawn(async move {

                    let (resp_tx, resp_rx) = oneshot::channel();

                    let cmd = IOEvent::LoadCustomViews { linear_config: linear_config,
                                                            linear_cursor: view_cursor,
                                                            resp: resp_tx };
                    tx2.send(cmd).await.unwrap();

                    let res = resp_rx.await.ok();

                    info!("LoadCustomViews IOEvent returned: {:?}", res);

                    let mut view_data_lock = view_data_handle.lock().unwrap();
                    let mut view_cursor_data_lock = view_cursor_handle.lock().unwrap();

                    let mut current_views = view_data_lock.clone();
                    let mut merged_views = false;

                    match res {
                        Some(x) => {
                            match x {
                                Some(y) => {
                                    match y["views"] {
                                        serde_json::Value::Array(_) => {
                                            // info!("Updating view_data_lock to: {:?}", y["views"]);
                                            // *view_data_lock = Some(y["views"].clone());

                                            // Append to existing list of Views
                                            match current_views {
                                                Some(mut view_data) => {
                                                    match view_data {
                                                        serde_json::Value::Array(ref mut view_vec) => {
                                                            view_vec.append(
                                                                &mut y["views"]
                                                                    .clone()
                                                                    .as_array_mut()
                                                                    .get_or_insert(&mut vec![]));
                                                            *view_data_lock = Some( serde_json::Value::Array(view_vec.clone()) );
                                                            merged_views = true;
                                                        },
                                                        _ => {},
                                                    }
                                                },
                                                _ => {}
                                            }

                                            if merged_views == false {
                                                *view_data_lock = Some( y["views"].clone());
                                            }
                                        },
                                        _ => {},
                                    };
                                    match GraphQLCursor::linear_cursor_from_page_info(y["cursor_info"].clone()) {
                                        Some(z) => {
                                            info!("Updating view_cursor_data_lock to: {:?}", z);
                                            *view_cursor_data_lock = z;
                                        },
                                        None => {},
                                    }
                                },
                                None => {

                                }
                            };
                        }
                        None => {},
                    }


                    info!("New self.linear_custom_view_select.view_table_data: {:?}", view_data_lock);
                });
            },

            "load_dashboard_views" => {
                // Reset app.linear_dashboard_view_panel_list
                let view_panel_list_ref = self.linear_dashboard_view_panel_list.clone();
                let mut view_panel_list_handle = view_panel_list_ref.lock().unwrap();

                view_panel_list_handle.clear();


                for filter in self.linear_dashboard_view_list.iter() {
                    match filter {
                        // Create DashboardViewPanels for each filter
                        Some(filter) => {
                            view_panel_list_handle.push(
                                components::dashboard_view_panel::DashboardViewPanel::with_filter(filter.clone())
                            );
                        },
                        None => {},
                    }
                }
                info!("change_route ActionSelect new self.linear_dashboard_view_panel_list: {:?}", view_panel_list_handle);

                // Create 'view_load_bundles': Vec<ViewLoadBundle> from view_panel_list_handle
                let view_load_bundles: Vec<ViewLoadBundle> = view_panel_list_handle
                                                                    .iter()
                                                                    .cloned()
                                                                    .map(|e| {
                                                                        ViewLoadBundle { linear_config: self.linear_client.config.clone(), 
                                                                                         item_filter: e.filter,
                                                                                         table_data: e.issue_table_data.clone(),
                                                                                         tx: tx.clone(),
                                                                                        }
                                                                    })
                                                                    .collect();



                drop(view_panel_list_handle);


                let t1 = tokio::spawn(async move {

                    // Load all DashboardViewPanels

                    /*
                    let mut iter_data: Vec<components::dashboard_view_panel::DashboardViewPanel> = Vec::new() 
                    {
                        let view_panel_lock = view_panel_list_ref.lock().unwrap();
                        iter_data = view_panel_lock.clone();
                    }
                    */



                    // note the use of `into_iter()` to consume `items`
                    let tasks: Vec<_> = view_load_bundles
                    .into_iter()
                    .map(|item| {
                        // item is: 
                        /*
                        pub struct DashboardViewPanel {
                            pub filter: Value,
                            pub issue_table_data: Arc<Mutex<Option<Value>>>,
                        }
                        */
                        info!("Spawning Get View Panel Issues Task");
                        // let tx2 = tx.clone();
                        // let temp_config = self.linear_client.config.clone();
                        // let view_panel_handle: Arc<_> = item.issue_table_data.clone();
                        // let item_filter = item.filter.clone();

                        tokio::spawn(async move {
                            let (resp_tx, resp_rx) = oneshot::channel();

                            let cmd = IOEvent::LoadViewIssues { linear_config: item.linear_config.clone(), view: item.item_filter.clone(),  resp: resp_tx };
                            item.tx.send(cmd).await.unwrap();
        
                            let res = resp_rx.await.ok();

                            info!("LoadViewIssues IOEvent returned: {:?}", res);

                            let mut view_panel_data_lock = item.table_data.lock().unwrap();

                            if let Some(x) = res {
                                *view_panel_data_lock = Some(Value::Array(x));
                            }
                            info!("New dashboard_view_panel.issue_table_data: {:?}", view_panel_data_lock);
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
                        info!("LoadViewIssues Result: {:?}", item);
                    }                    
                });

            },

            // Acquire these values to dispatch LoadLinearIssuesPaginate:
            //  linear_config: LinearConfig,
            //  linear_cursor: GraphQLCursor,
            //  selected_team: serde_json::Value,
            "load_issues_paginated" => {
                let tx2 = tx.clone();

                let linear_config = self.linear_client.config.clone();

                let linear_cursor_data_handle = self.linear_issue_cursor.lock().unwrap();
                let linear_cursor: GraphQLCursor = linear_cursor_data_handle.clone();
                drop(linear_cursor_data_handle);

                let team_issue_handle = self.linear_issue_display.issue_table_data.clone();
                let team_issue_cursor_handle = self.linear_issue_cursor.clone();

                let team_data_handle = self.linear_team_select.teams_data.clone();

                match self.linear_selected_team_idx {
                    Some(idx) => {
                        // Arc<Option<T>> -> Option<&T>
                        match &*team_data_handle.lock().unwrap() {
                            Some(data) => {
                                
                                let team = data[idx].clone();

                                match team {
                                    serde_json::Value::Null => return,
                                    _ => {},
                                }

                                let t1 = tokio::spawn(async move {

                                    let (resp_tx, resp_rx) = oneshot::channel();

                                    let cmd = IOEvent::LoadLinearIssuesPaginate { linear_config: linear_config,
                                                                                  linear_cursor: linear_cursor,
                                                                                  selected_team: team,
                                                                                  resp: resp_tx 
                                                                                };
                                    tx2.send(cmd).await.unwrap();

                                    let res = resp_rx.await.ok();

                                    info!("LoadLinearIssuesPaginate IOEvent returned: {:?}", res);

                                    let mut issue_data_lock = team_issue_handle.lock().unwrap();
                                    let mut issue_cursor_data_lock = team_issue_cursor_handle.lock().unwrap();
                                    let mut current_issues = issue_data_lock.clone();
                                    let mut merged_issues = false;

                                    match res {
                                        Some(x) => {
                                            match x {
                                                Some(y) => {
                                                    match y["issues"] {
                                                        serde_json::Value::Array(_) => {
                                                            // Append to existing list of Issues
                                                            match current_issues {
                                                                Some(mut issue_data) => {
                                                                    match issue_data {
                                                                        serde_json::Value::Array(ref mut issue_vec) => {
                                                                            issue_vec.append(
                                                                                &mut y["issues"]
                                                                                    .clone()
                                                                                    .as_array_mut()
                                                                                    .get_or_insert(&mut vec![]));
                                                                            *issue_data_lock = Some( serde_json::Value::Array(issue_vec.clone()) );
                                                                            merged_issues = true;
                                                                        },
                                                                        _ => {},
                                                                    }
                                                                },
                                                                _ => {}
                                                            }

                                                            if merged_issues == false {
                                                                *issue_data_lock = Some( y["issues"].clone());
                                                            }

                                                        },
                                                        _ => {},
                                                    };
                                                    match GraphQLCursor::linear_cursor_from_page_info(y["cursor_info"].clone()) {
                                                        Some(z) => {
                                                            info!("Updating issue_cursor_data_lock to: {:?}", z);
                                                            *issue_cursor_data_lock = z;
                                                        },
                                                        None => {},
                                                    }
                                                },
                                                None => {

                                                }
                                            };
                                        }
                                        None => {},
                                    }

                                });


                            },
                            None => {},
                        };
                    },
                    None => {return;}
                }
            },
            "load_workflows" => {
                let tx2 = tx.clone();

                let api_key = self.linear_client.config.api_key.clone();

                let team_data_handle = self.linear_team_select.teams_data.clone();

                let workflow_data_handle = self.linear_workflow_select.workflow_states_data.clone();


                match self.linear_selected_team_idx {
                    Some(idx) => {
                        // Arc<Option<T>> -> Option<&T>
                        match &*team_data_handle.lock().unwrap() {
                            Some(data) => {
                                
                                let team = data[idx].clone();

                                match team {
                                    serde_json::Value::Null => return,
                                    _ => {},
                                }

                                let t1 = tokio::spawn(async move {

                                    let (resp_tx, resp_rx) = oneshot::channel();

                                    let cmd = IOEvent::LoadWorkflowStates { api_key: api_key, selected_team: team, resp: resp_tx };
                                    tx2.send(cmd).await.unwrap();

                                    let res = resp_rx.await.ok();

                                    info!("LoadWorkflowStates IOEvent returned: {:?}", res);

                                    let mut workflow_data_lock = workflow_data_handle.lock().unwrap();

                                    match res {
                                        Some(x) => {
                                            *workflow_data_lock = x;
                                        }
                                        None => {},
                                    }
                                    info!("New self.linear_workflow_select.workflow_states_data: {:?}", workflow_data_lock);

                                });
                            }
                            None => {}
                        }
                    }
                    None => {return;}
                }
            },
            "update_issue_workflow" => {
                let tx3 = tx.clone();

                let api_key = self.linear_client.config.api_key.clone();

                // Need to get selected Workflow State and selected Issue
                let issue_data_handle = self.linear_issue_display.issue_table_data.clone();
                let workflow_state_data_handle = self.linear_workflow_select.workflow_states_data.clone();

                // Get Linear selected Issue index
                match self.linear_selected_issue_idx {
                    Some(issue_idx) => {
                        // Acquire a lock on Linear Issue data
                        match &*issue_data_handle.lock().unwrap() {
                            Some(issue_data) => {
                                // Get Linear selected Workflow State index
                                match self.linear_selected_workflow_state_idx {
                                    Some(workflow_idx) => {
                                        // Acquire a lock on Linear Workflow state data
                                        match &*workflow_state_data_handle.lock().unwrap() {
                                            Some(workflow_data) => {
                                                // Acquire relevant issue and workflow state
                                                let selected_issue = issue_data[issue_idx].clone();
                                                let selected_workflow_state = workflow_data[workflow_idx].clone();
                                                let mut issue_update_data_handle = self.linear_issue_display.issue_table_data.clone();

                                                // Spawn task to issue command to update workflow state
                                                let t3 = tokio::spawn( async move {
                                                    let (resp2_tx, resp2_rx) = oneshot::channel();

                                                    let cmd = IOEvent::UpdateIssueWorkflowState {   api_key: api_key,
                                                                                                    selected_issue: selected_issue.clone(),
                                                                                                    selected_workflow_state: selected_workflow_state.clone(),
                                                                                                    resp: resp2_tx  
                                                                                                };
                                                    tx3.send(cmd).await.unwrap();

                                                    let res = resp2_rx.await.ok();

                                                    info!("UpdateIssueWorkflowState IOEvent returned: {:?}", res);

                                                    // UpdateIssueWorkflowState IOEvent returned: Some(Some(Object({"issue_response": Object({"createdAt": String("2021-02-06T17:47:01.039Z"), "id": String("ace38e69-8a64-46f8-ad57-dc70c61f5599"), "number": Number(11), "title": String("Test Insomnia 1")}), "success": Bool(true)})))
                                                    // If Some(Some(Object({"success": Bool(true)})))
                                                    // then can match linear_issue_display.issue_table_data using selected_issue["id"]
                                                    // and update linear_issue_display.issue_table_data[x]["state"] with selected_workflow_state

                                                    let mut update_succeeded = false;
                                                    
                                                    match res {
                                                        Some(x) => {
                                                            match x {
                                                                Some(query_response) => {
                                                                    // update_succeeded = queryResponse["success"].as_bool().get_or_insert(false);
                                                                    if let serde_json::Value::Bool(value) = query_response["success"] {
                                                                        update_succeeded = value;//.as_bool().get_or_insert(false);
                                                                    }
                                                                },
                                                                None => {}
                                                            }
                                                        },
                                                        None => {},
                                                    }

                                                    let updated_issue_id = String::from(*selected_issue["id"].as_str().get_or_insert(""));// = selected_issue["id"];
                                                    /*
                                                    match &selected_issue["id"] {
                                                        serde_json::Value::String(x) => updated_issue_id = x,
                                                        _ => { update_succeeded = false }
                                                    };
                                                    */

                                                    // match linear_issue_display.issue_table_data using selected_issue["id"]
                                                    if update_succeeded == true && updated_issue_id.chars().count() > 0 {

                                                        let mut state = &mut *issue_update_data_handle.lock().unwrap();                                                        

                                                        
                                                        match state.as_mut() {// &*issue_update_data_handle.lock().unwrap() { 
                                                            Some(issue_update_target_data) =>  {
                                                                match issue_update_target_data.as_array_mut() {
                                                                    Some(table_array) => {
                                                                        let issue_to_update_option = table_array.iter()
                                                                                                                .position(|r| {
                                                                                                                                if let serde_json::Value::String(issue_id) = &r["id"] {
                                                                                                                                    if *issue_id == *updated_issue_id {
                                                                                                                                        return true;
                                                                                                                                    }
                                                                                                                                }
                                                                                                                                return false;
                                                                                                                });
                                                                        // Should be Some(x{0..})
                                                                        info!("issue_to_update_option: {:?}", issue_to_update_option);

                                                                        if let Some(issue_to_update_index) = issue_to_update_option {
                                                                            //table_array[issue_to_update_index]["state"] = selected_workflow_state;
                                                                            match table_array[issue_to_update_index].as_object_mut() {
                                                                                Some(issue_object_to_update) => {
                                                                                    issue_object_to_update["state"] = selected_workflow_state.clone();
                                                                                },
                                                                                None => {}
                                                                            }
                                                                        }
                                                                    }
                                                                    None => {}
                                                                }
                                                            }
                                                            None => {}
                                                        }
                                                        // Get index where: linear_issue_display.issue_table_data[index]["id"] == selected_issue["id"]


                                                    }

                                                });
                                            },
                                            None => {},
                                        }

                                    },
                                    None => {}
                                };
                            },
                            None => {},
                        };
                    },
                    None => {},
                }
            }
            _ => {return},
        }

    }

}