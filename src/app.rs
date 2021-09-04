use crate::util;
use crate::linear;
use crate::network;

use network::IOEvent as IOEvent;


use tokio::sync::oneshot;

use std::sync::{Arc, Mutex};

use crate::constants::{
    IssueModificationOp
};

use crate::linear::{
    LinearConfig,
    view_resolver::ViewLoader
};

use serde_json::Value;

use std::collections::{HashSet, HashMap};

use crate::util::{
    StatefulList as StatefulList,
    GraphQLCursor,
    dashboard::fetch_selected_view_panel_issue,
    dashboard::fetch_selected_value,
};

use crate::components::{
    command_bar::{ CommandBar, CommandBarType },

    linear_custom_view_select::LinearCustomViewSelect,

    dashboard_view_display::DashboardViewDisplay,
    dashboard_view_panel::DashboardViewPanel,

    linear_issue_op_interface::LinearIssueOpInterface,
};

use tui::{
    widgets::{ TableState },
};

pub struct ViewLoadBundle {
    pub linear_config: LinearConfig,

    pub tz_id_name_lookup: HashMap<String, String>,
    pub tz_name_offset_lookup: Arc<Mutex<HashMap<String, f64>>>,

    pub item_filter: Value,
    pub table_data: Arc<Mutex<Vec<Value>>>,
    pub loader: Arc<Mutex<Option<ViewLoader>>>,
    pub request_num: Arc<Mutex<u32>>,
    pub loading: Arc<Mutex<bool>>,

    pub tx: tokio::sync::mpsc::Sender<IOEvent>,
}

#[derive(PartialEq)]
pub enum Route {
    ActionSelect,
    DashboardViewDisplay
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
    pub linear_client: linear::client::LinearClient,

    // loader_tick is a looping index for loader_state
    pub loader_tick: u16,

    // TimeZone Manager
    pub tz_name_offset_map: Arc<Mutex<HashMap<String, f64>>>,

    pub team_tz_map: Arc<Mutex<HashMap<String, String>>>,
    pub team_tz_load_done: Arc<Mutex<bool>>,

    // Linear Custom View Select
    pub linear_custom_view_select: LinearCustomViewSelect,
    // Selected Custom View
    pub linear_selected_custom_view_idx: Option<usize>,
    // Linear Custom View Cursor
    pub linear_custom_view_cursor: Arc<Mutex<GraphQLCursor>>,

    // Linear Dashboard Custom View List Display
    pub dashboard_view_display: DashboardViewDisplay,
    pub dashboard_view_config_cmd_bar: CommandBar<'a>,

    // Linear Dashboard Custom View List
    pub linear_dashboard_view_list: Vec<Option<Value>>,
    pub linear_dashboard_view_idx: Option<usize>,
    pub linear_dashboard_view_list_selected: bool,

    // Linear Dashboard View Panel Display

    // Linear Dashboard 'DashboardViewPanel' components
    pub linear_dashboard_view_panel_list: Arc<Mutex<Vec<DashboardViewPanel>>>,
    pub linear_dashboard_view_panel_selected: Option<usize>,
    pub view_panel_issue_selected: Option<TableState>,
    pub view_panel_to_paginate: usize,

    pub view_panel_cmd_bar: CommandBar<'a>,


    // Issue Modification fields
    pub modifying_issue: bool,
    pub linear_issue_op_interface: LinearIssueOpInterface,

    // Available actions
    pub actions: StatefulList<&'a str>,
}



impl<'a> Default for App<'a> {
    fn default() -> App<'a> {
        App {
            route: Route::ActionSelect,
            cmd_str: String::new(),

            linear_client: linear::client::LinearClient::default(),

            loader_tick: 0,

            tz_name_offset_map: Arc::new(Mutex::new(linear::parse_timezones_from_file())),

            team_tz_map: Arc::new(Mutex::new(HashMap::new())),
            team_tz_load_done: Arc::new(Mutex::new(false)),

            linear_custom_view_select: LinearCustomViewSelect::default(),
            linear_selected_custom_view_idx: None,
            linear_custom_view_cursor: Arc::new(Mutex::new(GraphQLCursor::default())),

            dashboard_view_display: DashboardViewDisplay::default(),
            dashboard_view_config_cmd_bar: CommandBar::with_type(CommandBarType::ViewList),


            linear_dashboard_view_list: vec![ None, None, None, None, None, None ],
            linear_dashboard_view_idx: None,
            linear_dashboard_view_list_selected: true,

            linear_dashboard_view_panel_list: Arc::new(Mutex::new(Vec::with_capacity(6))),
            linear_dashboard_view_panel_selected: None,
            view_panel_issue_selected: None,
            view_panel_to_paginate: 0,

            view_panel_cmd_bar: CommandBar::with_type(CommandBarType::Dashboard),

            modifying_issue: false,
            linear_issue_op_interface: LinearIssueOpInterface::default(),

            actions: util::StatefulList::with_items(vec![
                "Modify Dashboard",
                "Create New Custom View",
            ]).selected(),
        }
    }
}







impl<'a> App<'a> {


    pub fn change_route(&mut self, route: Route, tx: &tokio::sync::mpsc::Sender<IOEvent>) {
        match route {

            // Create DashboardViewPanel components for each Some in app.linear_dashboard_view_list
            // and set app.linear_dashboard_view_panel_list
            // Load all Dashboard Views
            Route::ActionSelect => {
                self.dispatch_event("load_dashboard_views", &tx);
            },

            Route::DashboardViewDisplay => {

                /*
                // TODO: Clear any previous CustomViewSelect related values on self
                self.linear_custom_view_select = components::linear_custom_view_select::LinearCustomViewSelect::default();
                self.linear_selected_custom_view_idx = None;
                self.linear_custom_view_cursor = Arc::new(Mutex::new(GraphQLCursor::default()));

                self.dispatch_event("load_custom_views", tx);
                */

                // TODO: Clear any previous CustomViewSelect related values on self
                self.linear_custom_view_select = LinearCustomViewSelect::default();
                self.linear_selected_custom_view_idx = None;
                self.linear_custom_view_cursor = Arc::new(Mutex::new(GraphQLCursor::default()));

                self.linear_dashboard_view_list_selected = true;

                self.dispatch_event("load_custom_views", tx);
            }
        }
        self.route = route;
    }

    pub fn dispatch_event(&mut self, event_name: &str, tx: &tokio::sync::mpsc::Sender<IOEvent>) {

        match event_name {

            "load_custom_views" => {
                // TODO: Clear any previous CustomViewSelect related values on self


                let view_select_loading_handle = self.linear_custom_view_select.loading.clone();

                let mut view_select_loading_lock = view_select_loading_handle.lock().unwrap();

                // If already loading something, don't try again
                if *view_select_loading_lock {
                    return;
                }

                // Set Loading 'true' before fetch
                *view_select_loading_lock = true;
                drop(view_select_loading_lock);


                let tx2 = tx.clone();

                let linear_config = self.linear_client.config.clone();

                let view_data_handle = self.linear_custom_view_select.view_table_data.clone();


                let view_cursor_handle = self.linear_custom_view_cursor.lock().unwrap();
                let view_cursor: GraphQLCursor = view_cursor_handle.clone();
                drop(view_cursor_handle);

                let view_cursor_handle = self.linear_custom_view_cursor.clone();

                let _t1 = tokio::spawn(async move {

                    let (resp_tx, resp_rx) = oneshot::channel();

                    let cmd = IOEvent::LoadCustomViews { linear_config,
                                                            linear_cursor: view_cursor,
                                                            resp: resp_tx };
                    tx2.send(cmd).await.unwrap();

                    let res = resp_rx.await.ok();

                    info!("LoadCustomViews IOEvent returned: {:?}", res);

                    let mut view_data_lock = view_data_handle.lock().unwrap();
                    let mut view_cursor_data_lock = view_cursor_handle.lock().unwrap();
                    let mut view_select_loading_lock = view_select_loading_handle.lock().unwrap();

                    let mut current_views = view_data_lock.clone();

                    if let Some(Some(mut y)) = res {

                        if let Some(new_views_vec) = y["views"].as_array_mut() {
                            current_views.append(new_views_vec);
                            *view_data_lock = current_views;
                            *view_select_loading_lock = false;
                        }

                        match GraphQLCursor::linear_cursor_from_page_info(y["cursor_info"].clone()) {
                            Some(z) => {
                                info!("Updating view_cursor_data_lock to: {:?}", z);
                                *view_cursor_data_lock = z;
                            },
                            None => {
                                error!("'load_custom_views' linear_cursor_from_page_info() failed for cursor_info: {:?}", y["cursor_info"]);
                                panic!("'load_custom_views' linear_cursor_from_page_info() failed for cursor_info: {:?}", y["cursor_info"]);
                            },
                        }
                    }

                    info!("New self.linear_custom_view_select.view_table_data: {:?}", view_data_lock);
                });
            },

            "load_dashboard_views" => {
                // Reset app.linear_dashboard_view_panel_list
                let view_panel_list_ref = self.linear_dashboard_view_panel_list.clone();
                let mut view_panel_list_handle = view_panel_list_ref.lock().unwrap();

                // view_panel_list_handle.clear();

                let mut existing_panel_set = HashSet::new();

                debug!("dispatch_event::load_dashboard_views - self.linear_dashboard_view_list: {:?}", self.linear_dashboard_view_list);

                for (i, filter_opt) in self.linear_dashboard_view_list.iter().enumerate() {
                    //  If a View Panel for the filter is present within self.linear_dashboard_view_panel_list
                    //  and self.linear_dashboard_view_panel_list[x].is_loading == false,
                    //      if the index doesn't match:
                    //          clone the view panel and insert into the correct index within self.linear_dashboard_view_panel_list
                    //      else:
                    //          do not insert a new view panel

                    if let Some(filter) = filter_opt {
                        // Create DashboardViewPanels for each filter

                        let filter_id = filter["id"].clone();
                        let filter_view_panel_exists = view_panel_list_handle
                                                        .iter()
                                                        .position(|e| { 
                                                            debug!("filter_view_panel_exists comparing {:?} == {:?}", e.filter["id"], filter_id);   
                                                            e.filter["id"] == filter_id
                                                        });
                        debug!("i: {:?}, filter_view_panel_exists: {:?}", i, filter_view_panel_exists);


                        match filter_view_panel_exists {
                            Some(filter_view_panel_idx) => {

                                //  if the index doesn't match:
                                //      clone the view panel and replace into the correct index
                                //      within self.linear_dashboard_view_panel_list

                                if i != filter_view_panel_idx {
                                    let dup_view_panel = view_panel_list_handle[filter_view_panel_idx].clone();
                                    // view_panel_list_handle.insert(i, dup_view_panel);
                                    if i < view_panel_list_handle.len() {
                                        let _got = std::mem::replace(&mut view_panel_list_handle[i], dup_view_panel);
                                    }
                                    else {
                                        view_panel_list_handle.insert(i, dup_view_panel);
                                    }
                                }

                                // TODO: Why is this not in an else?
                                // if the index does match, then a ViewPanel already exists for this filter, skip
                                existing_panel_set.insert(i);

                            },
                            // Need to create a new View Panel
                            None => {
                                debug!("Attempting to use insert for i: {:?}", i);
                                // view_panel_list_handle.insert(i, DashboardViewPanel::with_filter(filter.clone()));
                                // let got = std::mem::replace(&mut view_panel_list_handle[i], DashboardViewPanel::with_filter(filter.clone()));

                                if i < view_panel_list_handle.len() {
                                    let _got = std::mem::replace(&mut view_panel_list_handle[i], DashboardViewPanel::with_filter(filter.clone()));
                                }
                                else {
                                    view_panel_list_handle.insert(i, DashboardViewPanel::with_filter(filter.clone()));
                                }
                            }
                        };
                    }
                }

                info!("change_route ActionSelect new self.linear_dashboard_view_panel_list: {:?}", view_panel_list_handle);

                // Create 'view_load_bundles': Vec<ViewLoadBundle> from view_panel_list_handle
                // Filter to only create ViewLoadBundles for ViewPanels where 
                let view_load_bundles: Vec<ViewLoadBundle> = view_panel_list_handle
                    .iter()
                    .cloned()
                    .enumerate()
                    .filter_map(|(i, e)| {
                        if existing_panel_set.contains(&i) {
                            None
                        }
                        else {
                            Some(ViewLoadBundle {
                                            linear_config: self.linear_client.config.clone(),

                                            tz_id_name_lookup: self.team_tz_map.lock()
                                                                                .unwrap()
                                                                                .clone(),
                                            tz_name_offset_lookup: self.tz_name_offset_map.clone(),
                                            
                                            item_filter: e.filter,
                                            table_data: e.issue_table_data.clone(),
                                            loader: e.view_loader.clone(),
                                            request_num: e.request_num.clone(),
                                            loading: e.loading.clone(),

                                            tx: tx.clone(),
                                        })
                        }
                    })
                    .collect();



                drop(view_panel_list_handle);


                let _t1 = tokio::spawn(async move {

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

                        let loader_handle = item.loader.lock().unwrap();
                        let loader = loader_handle.clone();
                        drop(loader_handle);

                        // Set ViewPanel loading state to true
                        let mut loading_init_lock = item.loading.lock().unwrap();
                        *loading_init_lock = true;
                        drop(loading_init_lock);

                        tokio::spawn(async move {
                            let (resp_tx, resp_rx) = oneshot::channel();


                            let cmd = IOEvent::LoadViewIssues { linear_config: item.linear_config.clone(),
                                                                team_tz_lookup: item.tz_id_name_lookup,
                                                                tz_offset_lookup: item.tz_name_offset_lookup,
                                                                issue_data: Arc::new(Mutex::new(Vec::new())),
                                                                view: item.item_filter.clone(), 
                                                                view_loader: loader,
                                                                resp: resp_tx };
                            
                            item.tx.send(cmd).await.unwrap();
        
                            let res = resp_rx.await.ok();

                            info!("LoadViewIssues IOEvent returned: {:?}", res);

                            let mut view_panel_data_lock = item.table_data.lock().unwrap();
                            let mut loader_handle = item.loader.lock().unwrap();
                            let mut request_num_lock = item.request_num.lock().unwrap();
                            let mut loading_lock = item.loading.lock().unwrap();

                            if let Some(x) = res {
                                *view_panel_data_lock = x.0;
                                *loader_handle = Some(x.1);
                                *request_num_lock += x.2;
                                *loading_lock = false;
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
            "paginate_dashboard_view" => {

                let tx2 = tx.clone();

                let view_panel_list_handle = self.linear_dashboard_view_panel_list.lock().unwrap();

                let mut loading_init_lock = view_panel_list_handle[self.view_panel_to_paginate].loading.lock().unwrap();

                // If already loading something, don't try again
                if *loading_init_lock {
                    return;
                }

                // Set ViewPanel loading state to true
                *loading_init_lock = true;
                drop(loading_init_lock);


                let config = self.linear_client.config.clone();

                let view_panel_view_obj = view_panel_list_handle[self.view_panel_to_paginate].filter.clone();

                let loader_lock = view_panel_list_handle[self.view_panel_to_paginate].view_loader.lock().unwrap();
                let loader = loader_lock.clone();

                let view_panel_issue_handle = view_panel_list_handle[self.view_panel_to_paginate].issue_table_data.clone();
                let loader_handle = view_panel_list_handle[self.view_panel_to_paginate].view_loader.clone();
                let request_num_handle = view_panel_list_handle[self.view_panel_to_paginate].request_num.clone();


                let loading_handle = view_panel_list_handle[self.view_panel_to_paginate].loading.clone();

                let tz_id_name_lookup_dup = self.team_tz_map.lock()
                                                            .unwrap()
                                                            .clone();
                let tz_name_offset_lookup_dup = self.tz_name_offset_map.clone();


                drop(loader_lock);
                drop(view_panel_list_handle);


                let _t1 = tokio::spawn(async move {
                    let (resp_tx, resp_rx) = oneshot::channel();


                    let cmd = IOEvent::LoadViewIssues { linear_config: config,
                                                        team_tz_lookup: tz_id_name_lookup_dup,
                                                        tz_offset_lookup: tz_name_offset_lookup_dup,
                                                        issue_data: view_panel_issue_handle.clone(),
                                                        view: view_panel_view_obj, 
                                                        view_loader: loader,
                                                        resp: resp_tx };
                    
                    tx2.send(cmd).await.unwrap();

                    let res = resp_rx.await.ok();

                    info!("LoadViewIssues IOEvent returned: {:?}", res);
                    
                    let mut view_panel_data_lock = view_panel_issue_handle.lock().unwrap();
                    let mut loader = loader_handle.lock().unwrap();
                    let mut request_num_lock = request_num_handle.lock().unwrap();

                    let mut loading_lock = loading_handle.lock().unwrap();

                    let mut current_view_issues = view_panel_data_lock.clone();

                    if let Some(mut x) = res {

                        current_view_issues.append(&mut x.0);
                        *view_panel_data_lock = current_view_issues.clone();
                        *loader = Some(x.1);
                        *request_num_lock += x.2;
                        *loading_lock = false;

                    }
                    info!("New dashboard_view_panel.issue_table_data: {:?}", view_panel_data_lock);
                });
            },
            "load_issue_op_data" => {
                let tx2 = tx.clone();

                let op_interface_loading_handle = self.linear_issue_op_interface.loading.clone();
                let mut op_interface_loading_lock = op_interface_loading_handle.lock().unwrap();
                // If already loading something, don't try again
                if *op_interface_loading_lock {
                    return;
                }
                // Set Loading 'true' before fetch
                *op_interface_loading_lock = true;
                drop(op_interface_loading_lock);

                let issue_op_data_handle = self.linear_issue_op_interface.table_data_from_op();
                let linear_config = self.linear_client.config.clone();
                let current_op = self.linear_issue_op_interface.current_op;

                let selected_issue_opt = fetch_selected_view_panel_issue(&self);
                let selected_issue;
                let selected_team;

                // Check that an Issue is selected, if not return
                if let Some(x) = selected_issue_opt {
                    selected_issue = x;
                }
                else {
                    return;
                }

                // Get the Issue's team,
                // panic if not found since every Issue should have a value for ['team']['id']
                selected_team = selected_issue["team"]["id"].clone();

                if selected_team.is_null() {
                    error!("['team']['id'] returned Value::Null for Issue: {:?}", selected_issue);
                    panic!("['team']['id'] returned Value::Null for Issue: {:?}", selected_issue);
                }

                // Get Cursor
                let issue_op_cursor_lock = self.linear_issue_op_interface.cursor.lock().unwrap();
                let issue_op_cursor: GraphQLCursor = issue_op_cursor_lock.clone();
                drop(issue_op_cursor_lock);

                let issue_op_cursor_handle = self.linear_issue_op_interface.cursor.clone();


                let _t1 = tokio::spawn(async move {

                    let (resp_tx, resp_rx) = oneshot::channel();

                    debug!("Dispatching Load-{:?} event", current_op);

                    let cmd = match current_op {
                        IssueModificationOp::ModifyWorkflowState => {
                            IOEvent::LoadWorkflowStates { linear_config, linear_cursor: issue_op_cursor, team: selected_team, resp: resp_tx }
                        },
                        IssueModificationOp::ModifyAssignee => {
                            IOEvent::LoadTeamMembers { linear_config, linear_cursor: issue_op_cursor, team: selected_team, resp: resp_tx }
                        },
                        _ => {
                            error!("load_issue_op_data - invalid IssueModificationOp: {:?}", current_op);
                            panic!("load_issue_op_data - invalid IssueModificationOp: {:?}", current_op);
                        }
                    };

                    tx2.send(cmd).await.unwrap();

                    let mut res = resp_rx.await.ok();

                    let mut issue_op_cursor_data_lock = issue_op_cursor_handle.lock().unwrap();
                    let mut loading_lock = op_interface_loading_handle.lock().unwrap();
                    *loading_lock = false;


                    info!("Load-{:?} IOEvent returned: {:?}", current_op, res);

                    let mut issue_op_data_lock = issue_op_data_handle.lock().unwrap();

                    let mut current_issue_op_data = issue_op_data_lock.clone();

                    if let Some(Some(ref mut x)) = res {
                        debug!("x - {:?}", x);
                        if let Some(values_vec) = x["data"].as_array_mut() {
                            current_issue_op_data.append(&mut values_vec.to_vec());
                            *issue_op_data_lock = current_issue_op_data;
                        }

                        match GraphQLCursor::linear_cursor_from_page_info(x["cursor_info"].clone()) {
                            Some(z) => {
                                info!("Updating issue_op_cursor_data_lock to: {:?}", z);
                                *issue_op_cursor_data_lock = z;
                            },
                            None => {
                                error!("'load_issue_op_data' linear_cursor_from_page_info() failed for cursor_info: {:?}", x["cursor_info"]);
                                panic!("'load_issue_op_data' linear_cursor_from_page_info() failed for cursor_info: {:?}", x["cursor_info"]);
                            },
                        }
                    }

                    // info!("New self.linear_workflow_select.workflow_states_data: {:?}", workflow_data_lock);
                });
            }
            "update_issue" => {
                let tx3 = tx.clone();

                let issue_id: String;
                let selected_value_id: String;
                let value_obj;

                // Get relevant issue and selected Value id, return if anything not found
                {
                    let selected_issue_opt = fetch_selected_view_panel_issue(&self);
                    let issue_obj = if let Some(x) = selected_issue_opt { x } else { return; };
                    let issue_id_opt = issue_obj["id"].as_str();

                    let selected_value_opt = fetch_selected_value(&self);
                    value_obj = if let Some(x) = selected_value_opt { x } else { return; };
                    let value_id_opt = value_obj["id"].as_str();

                    if let Some(x) = issue_id_opt {
                        issue_id = String::from(x);
                    }
                    else {
                        return;
                    }

                    if let Some(x) = value_id_opt {
                        selected_value_id = String::from(x);
                    }
                    else {
                        return;
                    }
                }

                debug!("update_issue - issue_id, selected_value_id: {:?}, {:?}", issue_id, selected_value_id);

                let linear_config = self.linear_client.config.clone();
                let view_panel_list_arc = self.linear_dashboard_view_panel_list.clone();

                let current_op = self.linear_issue_op_interface.current_op.clone();

                // Spawn task to issue command to update workflow state
                let _t3 = tokio::spawn( async move {
                    let (resp2_tx, resp2_rx) = oneshot::channel();

                    let cmd = match current_op {
                        IssueModificationOp::ModifyWorkflowState => {
                            IOEvent::UpdateIssueWorkflowState {   linear_config,
                                issue_id: issue_id.clone(),
                                workflow_state_id: selected_value_id,
                                resp: resp2_tx  
                            }
                        },
                        IssueModificationOp::ModifyAssignee => {
                            IOEvent::UpdateIssueAssignee {   linear_config,
                                issue_id: issue_id.clone(),
                                assignee_id: selected_value_id,
                                resp: resp2_tx  
                            }
                        },
                        _ => {
                            error!("IssueModificationOp not supported for 'update_issue': {:?}", current_op);
                            panic!("Not ready");
                        }
                    };

                    tx3.send(cmd).await.unwrap();

                    let res = resp2_rx.await.ok();
                    
                    info!("UpdateIssue IOEvent returned: {:?}", res);

                    // UpdateIssueWorkflowState IOEvent returned: Some(Some(Object({"issue_response": Object({"createdAt": String("2021-02-06T17:47:01.039Z"), "id": String("ace38e69-8a64-46f8-ad57-dc70c61f5599"), "number": Number(11), "title": String("Test Insomnia 1")}), "success": Bool(true)})))
                    // If Some(Some(Object({"success": Bool(true)})))
                    // then can match linear_issue_display.issue_table_data using selected_issue["id"]
                    // and update linear_issue_display.issue_table_data[x]["state"] with selected_workflow_state

                    let mut update_succeeded = false;

                    if let Some(Some(query_response)) = res {
                        if let Value::Bool(value) = query_response["success"] {
                            update_succeeded = value;
                        }
                    }
                    

                    
                    // If update succeeded, iterate over all Issues in all ViewPanels
                    // and set issue["state"] = state_obj 
                    //     where id matches 'issue_id'
                    if update_succeeded {
                        let view_panel_list_handle = view_panel_list_arc.lock().unwrap();
                        for view_panel in view_panel_list_handle.iter() {

                            // Iterate over ViewPanel Issues
                            let mut issue_list_handle = view_panel.issue_table_data.lock().unwrap();

                            for issue_obj in issue_list_handle.iter_mut() {
                                if let Some(panel_issue_id) = issue_obj["id"].as_str() {
                                    if panel_issue_id == issue_id.as_str() {
                                        match current_op {
                                            IssueModificationOp::ModifyWorkflowState => {issue_obj["state"] = value_obj.clone();},
                                            IssueModificationOp::ModifyAssignee => {issue_obj["assignee"] = value_obj.clone();},
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            },

            _ => {},
        }

    }

}