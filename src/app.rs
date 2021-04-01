use crate::util;
use crate::linear;
use crate::components;
use crate::command;

use command::Command as Command;

use util::StatefulList as StatefulList;
use util::GraphQLCursor;

use tokio::sync::oneshot;

use std::sync::{Arc, Mutex};


pub enum Route {
    ActionSelect,
    TeamSelect,
    LinearInterface,
}

#[derive(Debug)]
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
    /// Current value of the input box
    input: String,
    // LinearClient
    linear_client: linear::client::LinearClient,

    // Linear Team Select State
    pub linear_team_select: components::linear_team_select::LinearTeamSelectState,

    // Selected Linear Team
    pub linear_selected_team_idx: Option<usize>,

    // Linear Issue Display State
    pub linear_issue_display: components::linear_issue_display::LinearIssueDisplayState,

    // Selected Linear Issue
    pub linear_selected_issue_idx: Option<usize>,

    // Linear Issue Display Cursor
    pub linear_issue_cursor: Arc<Mutex<util::GraphQLCursor>>,

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
            input: String::new(),

            linear_client: linear::client::LinearClient::default(),

            linear_team_select: components::linear_team_select::LinearTeamSelectState::default(),
            // Null
            linear_selected_team_idx: None,
 
            linear_issue_display: components::linear_issue_display::LinearIssueDisplayState::default(),
            linear_selected_issue_idx: None,
            linear_issue_cursor: Arc::new(Mutex::new(util::GraphQLCursor::platform_cursor(Platform::Linear))),

            
            linear_workflow_select: components::linear_workflow_state_display::LinearWorkflowStateDisplayState::default(),
            linear_selected_workflow_state_idx: None,

            linear_draw_workflow_state_select: false,

            actions: util::StatefulList::with_items(vec![
                "Create Issue",
                "Test",
            ]),
        }
    }
}







impl<'a> App<'a> {


    pub fn draw_issue_state_select(&self, platform: Platform) -> &bool {
        match platform {
            Linear => { return &self.linear_draw_workflow_state_select },
            Github => { return &false },
        }
    }

    pub fn set_draw_issue_state_select(&mut self, platform: Platform, v: bool) {
        match platform {
            Linear => { self.linear_draw_workflow_state_select = v },
            Github => { },
        };
    }

    pub async fn change_route(&mut self, route: Route, tx: &tokio::sync::mpsc::Sender<Command>) {
        match route {
            Route::ActionSelect => {},
            Route::TeamSelect => {

                let tx2 = tx.clone();

                let api_key = self.linear_client.config.api_key.clone();

                let team_data_handle = self.linear_team_select.teams_data.clone();


                let t1 = tokio::spawn(async move {

                    let (resp_tx, resp_rx) = oneshot::channel();

                    let cmd = Command::LoadLinearTeams { api_key: api_key, resp: resp_tx };
                    tx2.send(cmd).await.unwrap();

                    let res = resp_rx.await.ok();

                    info!("LoadLinearTeams Command returned: {:?}", res);

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
                
                                    let cmd = Command::LoadLinearIssues { linear_config: linear_config, selected_team: team, resp: resp2_tx };
                                    tx3.send(cmd).await.unwrap();
                
                                    let res = resp2_rx.await.ok();

                                    info!("LoadLinearIssues Command returned: {:?}", res);

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

            _ => {},
        }
        self.route = route;
    }

    pub fn dispatch_event(&mut self, event_name: &str, tx: &tokio::sync::mpsc::Sender<Command>) {

        match event_name {
            "paginate_issue_list" => {

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

                                    let cmd = Command::LoadWorkflowStates { api_key: api_key, selected_team: team, resp: resp_tx };
                                    tx2.send(cmd).await.unwrap();

                                    let res = resp_rx.await.ok();

                                    info!("LoadWorkflowStates Command returned: {:?}", res);

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

                                                    let cmd = Command::UpdateIssueWorkflowState {   api_key: api_key,
                                                                                                    selected_issue: selected_issue.clone(),
                                                                                                    selected_workflow_state: selected_workflow_state.clone(),
                                                                                                    resp: resp2_tx  
                                                                                                };
                                                    tx3.send(cmd).await.unwrap();

                                                    let res = resp2_rx.await.ok();

                                                    info!("UpdateIssueWorkflowState Command returned: {:?}", res);

                                                    // UpdateIssueWorkflowState Command returned: Some(Some(Object({"issue_response": Object({"createdAt": String("2021-02-06T17:47:01.039Z"), "id": String("ace38e69-8a64-46f8-ad57-dc70c61f5599"), "number": Number(11), "title": String("Test Insomnia 1")}), "success": Bool(true)})))
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