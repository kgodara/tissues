use crate::util;

use tokio::{
    sync::Mutex as tMutex,
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use crate::constants::IssueModificationOp;

use crate::linear::{
    client::{LinearClient, IssueFieldObject, IssueFieldResponse},
    schema::{
        CustomView, Issue, IssueUpdateInput, Viewer,
    },
    LinearConfig,
};

use std::collections::{HashSet};

use crate::util::{
    dashboard::fetch_selected_value, dashboard::fetch_selected_view_panel_issue, GraphQLCursor,
    StatefulList,
};

use crate::components::{
    command_bar::{CommandBar, CommandBarType},
    dashboard_view_config_display::DashboardViewConfigDisplay,
    dashboard_view_panel::DashboardViewPanel,
    linear_custom_view_select::LinearCustomViewSelect,
    linear_issue_op_interface::LinearIssueOpInterface,

    token_entry::{ TokenEntry, TokenValidationState },
    title_entry::{ TitleEntry },

    InputComponent,
};

use tui::widgets::TableState;

pub struct ViewLoadBundle {
    pub linear_client: Arc<tMutex<Option<LinearClient>>>,

    pub item_filter: CustomView,
    pub table_data: Arc<Mutex<Vec<Issue>>>,
    pub cursor: Arc<Mutex<Option<GraphQLCursor>>>,
    pub loading: Arc<AtomicBool>,
}

#[derive(PartialEq, Clone)]
pub enum Route {
    ConfigInterface,
    ActionSelect,
    DashboardViewDisplay,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Edit,
}

pub enum AppEvent {
    LoadViewer,
    LoadCustomViews,
    LoadDashboardViews,
    PaginateDashboardView,
    LoadIssueOpData,
    UpdateIssue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    Na,
    Linear,
    Github,
}

// App holds the state of the application
pub struct App<'a> {
    // current route
    pub route: Arc<Mutex<Route>>,
    pub change_route: Arc<AtomicBool>,

    /// Current value of the Command string
    pub cmd_str: String,
    // LinearClient
    pub linear_client: Arc<tMutex<Option<LinearClient>>>,

    // Current input mode
    pub input_mode: InputMode,

    // Active Input
    pub active_input: InputComponent,

    // Token Entry Input Component
    pub token_entry: TokenEntry,

    // Issue Title Entry Input Component
    pub title_entry: TitleEntry,

    // loader_tick is a looping index for loader_state
    pub loader_tick: u16,

    // scroll_tick is an index which loops over 100 for paragraph scrolling
    pub scroll_tick: u64,

    // has previously cached view list been checked for
    pub view_list_cache_read_attempted: bool,

    // Viewer Object for rendering (can't use instance under LinearConfig because tokio Mutex requires await for lock acquisition)
    pub viewer_obj_render: Arc<Mutex<Option<Viewer>>>,

    // Linear Custom View Select
    pub linear_custom_view_select: LinearCustomViewSelect,
    // Selected Custom View
    pub linear_selected_custom_view_idx: Option<usize>,
    // Linear Custom View Cursor
    pub linear_custom_view_cursor: Arc<Mutex<GraphQLCursor>>,

    // Linear Dashboard Custom View List Display
    pub dashboard_view_display: DashboardViewConfigDisplay,
    pub dashboard_view_config_cmd_bar: CommandBar<'a>,

    // Linear Dashboard Custom View List
    pub linear_dashboard_view_list: Vec<Option<CustomView>>,
    pub linear_dashboard_view_idx: Option<usize>,
    pub linear_dashboard_view_list_selected: bool,

    // Linear Dashboard View Panel Display

    // Linear Dashboard 'DashboardViewPanel' components
    pub linear_dashboard_view_panel_list: Arc<Mutex<Vec<DashboardViewPanel>>>,
    pub linear_dashboard_view_panel_selected: Option<usize>,
    pub view_panel_issue_selected: Option<TableState>,
    pub view_panel_to_paginate: usize,

    pub view_panel_cmd_bar: CommandBar<'a>,

    pub issue_to_expand: Option<Issue>,

    // Issue Modification fields
    pub modifying_issue: bool,
    pub linear_issue_op_interface: LinearIssueOpInterface,

    // Available actions
    pub actions: StatefulList<&'a str>,
}

impl<'a> Default for App<'a> {
    fn default() -> App<'a> {
        App {
            route: Arc::new(Mutex::new(Route::ConfigInterface)),
            change_route: Arc::new(AtomicBool::new(false)),

            cmd_str: String::new(),

            linear_client: Arc::new(tMutex::new(None)),

            input_mode: InputMode::Normal,
            active_input: InputComponent::TokenEntry,

            token_entry: TokenEntry::default(),
            title_entry: TitleEntry::default(),

            // access_token_to_validate: String::from(""),
            loader_tick: 0,
            scroll_tick: 0,

            view_list_cache_read_attempted: false,

            viewer_obj_render: Arc::new(Mutex::new(None)),

            linear_custom_view_select: LinearCustomViewSelect::default(),
            linear_selected_custom_view_idx: None,
            linear_custom_view_cursor: Arc::new(Mutex::new(GraphQLCursor::with_platform(Platform::Linear))),

            dashboard_view_display: DashboardViewConfigDisplay::default(),
            dashboard_view_config_cmd_bar: CommandBar::with_type(CommandBarType::ViewList),

            linear_dashboard_view_list: vec![None, None, None, None, None, None],
            linear_dashboard_view_idx: None,
            linear_dashboard_view_list_selected: true,

            linear_dashboard_view_panel_list: Arc::new(Mutex::new(Vec::with_capacity(6))),
            linear_dashboard_view_panel_selected: None,
            view_panel_issue_selected: None,
            view_panel_to_paginate: 0,

            view_panel_cmd_bar: CommandBar::with_type(CommandBarType::Dashboard),

            issue_to_expand: None,

            modifying_issue: false,
            linear_issue_op_interface: LinearIssueOpInterface::default(),

            actions: util::StatefulList::with_items(vec!["Modify Dashboard"]).selected(),
        }
    }
}

impl<'a> App<'a> {
    pub fn change_route(&mut self, route: Route) {
        match route {
            Route::ConfigInterface => {
                // currently, config interface always has editor availability
                self.input_mode = InputMode::Edit;
                self.active_input = InputComponent::TokenEntry;

                // Unselect from actions list
                self.actions.unselect();
            }

            // Create DashboardViewPanel components for each Some in app.linear_dashboard_view_list
            // and set app.linear_dashboard_view_panel_list
            // Load all Dashboard Views
            Route::ActionSelect => {
                // no editor available
                self.input_mode = InputMode::Normal;

                // Select first action
                self.actions.next();

                if !self.view_list_cache_read_attempted {
                    let cached_read_option = LinearConfig::read_view_list();
                    if let Some(cached_view_list) = cached_read_option {
                        self.linear_dashboard_view_list = cached_view_list;
                    }
                }

                self.dispatch_event(AppEvent::LoadDashboardViews);
            }

            Route::DashboardViewDisplay => {
                // no editor available
                self.input_mode = InputMode::Normal;

                // Unselect from actions list
                self.actions.unselect();

                // Clear any previous CustomViewSelect related values on self
                self.linear_custom_view_select = LinearCustomViewSelect::default();
                self.linear_selected_custom_view_idx = None;
                self.linear_custom_view_cursor = Arc::new(Mutex::new(GraphQLCursor::with_platform(Platform::Linear)));

                self.linear_dashboard_view_list_selected = true;

                self.dispatch_event(AppEvent::LoadCustomViews);
            }
        }
        *self.route.lock().unwrap() = route;
    }

    pub fn dispatch_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::LoadViewer => {
                let token_validation_state_handle =
                    self.token_entry.token_validation_state.clone();
                {
                    let mut token_validation_state_lock =
                        token_validation_state_handle.lock().unwrap();
                    *token_validation_state_lock = TokenValidationState::Validating;
                }

                let token: String = self.token_entry.input.input.clone();

                let linear_client_handle = self.linear_client.clone();

                let route_handle = self.route.clone();
                let change_route_handle = self.change_route.clone();

                let viewer_obj_render_handle = self.viewer_obj_render.clone();

                let _t1 = tokio::spawn(async move {
                    // Temporary client without caching
                    let temp_client = LinearClient::with_config(LinearConfig::new(&token, None,false)).unwrap();

                    let res = temp_client.viewer().await;

                    debug!("AppEvent::LoadViewer - res: {res:?}");

                    // Check for "errors" field, if not found save access token
                    if let Ok(Some(resp_data)) = res {
                        {

                            let mut linear_client_lock = linear_client_handle.lock().await;

                            *linear_client_lock = Some(
                                LinearClient::with_config(LinearConfig::new(&token, Some(resp_data.viewer.clone()),true)).unwrap()
                            );

                            *route_handle.lock().unwrap() = Route::ActionSelect;
                            change_route_handle.store(true, Ordering::Relaxed);

                            let mut viewer_obj_render_lock = viewer_obj_render_handle.lock().unwrap();
                            *viewer_obj_render_lock = Some(resp_data.viewer.clone());

                            let mut token_validation_state_lock = token_validation_state_handle.lock().unwrap();
                            *token_validation_state_lock = TokenValidationState::Valid;
                        }
                    } else {
                        let mut token_validation_state_lock = token_validation_state_handle.lock().unwrap();
                        *token_validation_state_lock = TokenValidationState::Invalid;
                    }
                });
            }

            AppEvent::LoadCustomViews => {
                // TODO: Clear any previous CustomViewSelect related values on self

                let view_select_loading_handle = self.linear_custom_view_select.loading.clone();
                // If already loading something, don't try again
                if view_select_loading_handle.load(Ordering::Relaxed) {
                    return;
                }
                // Set Loading 'true' before fetch
                view_select_loading_handle.store(true, Ordering::Relaxed);

                let linear_client_handle = self.linear_client.clone();

                let view_data_handle = self.linear_custom_view_select.view_table_data.clone();

                let view_cursor_handle = self.linear_custom_view_cursor.lock().unwrap();
                let view_cursor: GraphQLCursor = view_cursor_handle.clone();
                drop(view_cursor_handle);

                let view_cursor_handle = self.linear_custom_view_cursor.clone();

                let _t1 = tokio::spawn(async move {
                    let linear_client_lock = linear_client_handle.lock().await;
                    let client = linear_client_lock.as_ref().unwrap();

                    let res  = client.custom_views(Some(view_cursor)).await;

                    let mut view_data_lock = view_data_handle.lock().unwrap();
                    let mut view_cursor_data_lock = view_cursor_handle.lock().unwrap();

                    let mut current_views = view_data_lock.clone();

                    if let Ok(Some(mut y)) = res {
                        current_views.append(&mut y.custom_views.nodes);
                        *view_data_lock = current_views;
                        view_select_loading_handle.store(false, Ordering::Relaxed);

                        // Update GraphQLCursor
                        // TODO: change unwrap_or
                        *view_cursor_data_lock = GraphQLCursor {
                            platform: Platform::Linear,
                            has_next_page: y.custom_views.page_info.has_next_page,
                            end_cursor: y
                                .custom_views
                                .page_info
                                .end_cursor,
                        };
                    } else {
                        error!("LoadCustomViews error: {:?}", res);
                        panic!("LoadCustomViews error: {:?}", res);
                    }

                    info!(
                        "New self.linear_custom_view_select.view_table_data: {:?}",
                        view_data_lock
                    );
                });
            }

            AppEvent::LoadDashboardViews => {
                // Reset app.linear_dashboard_view_panel_list
                let view_panel_list_handle = self.linear_dashboard_view_panel_list.clone();
                let mut view_panel_list_lock = view_panel_list_handle.lock().unwrap();

                // view_panel_list_handle.clear();

                let mut existing_panel_set = HashSet::new();

                debug!(
                    "dispatch_event::load_dashboard_views - self.linear_dashboard_view_list: {:?}",
                    self.linear_dashboard_view_list
                );

                for (i, view_opt) in self.linear_dashboard_view_list.iter().enumerate() {
                    //  If a View Panel for the filter is present within self.linear_dashboard_view_panel_list
                    //  and self.linear_dashboard_view_panel_list[x].is_loading == false,
                    //      if the index doesn't match:
                    //          clone the view panel and insert into the correct index within self.linear_dashboard_view_panel_list
                    //      else:
                    //          do not insert a new view panel

                    if let Some(view) = view_opt {
                        // Create DashboardViewPanels for each filter

                        let view_id = view.id.clone();
                        let custom_view_view_panel_exists =
                            view_panel_list_lock.iter().position(|e| {
                                debug!(
                                    "filter_view_panel_exists comparing {:?} == {:?}",
                                    e.view.id, view_id
                                );
                                e.view.id == view_id
                            });

                        match custom_view_view_panel_exists {
                            Some(filter_view_panel_idx) => {
                                //  if the index doesn't match:
                                //      clone the view panel and replace into the correct index
                                //      within self.linear_dashboard_view_panel_list
                                //  if the index does match:
                                //      then a ViewPanel already exists for this filter

                                if i != filter_view_panel_idx {
                                    let dup_view_panel =
                                        view_panel_list_lock[filter_view_panel_idx].clone();
                                    if i < view_panel_list_lock.len() {
                                        let _got = std::mem::replace(
                                            &mut view_panel_list_lock[i],
                                            dup_view_panel,
                                        );
                                    } else {
                                        view_panel_list_lock.insert(i, dup_view_panel);
                                    }
                                }

                                existing_panel_set.insert(i);
                            }
                            // Need to create a new View Panel
                            None => {
                                if i < view_panel_list_lock.len() {
                                    let _got = std::mem::replace(
                                        &mut view_panel_list_lock[i],
                                        DashboardViewPanel::with_view(view.clone()),
                                    );
                                } else {
                                    view_panel_list_lock
                                        .insert(i, DashboardViewPanel::with_view(view.clone()));
                                }
                            }
                        };
                    }
                }

                // Create 'view_load_bundles': Vec<ViewLoadBundle> from view_panel_list_handle
                // Filter to only create ViewLoadBundles for ViewPanels where
                let view_load_bundles: Vec<ViewLoadBundle> = view_panel_list_lock
                    .iter()
                    .cloned()
                    .enumerate()
                    .filter_map(|(i, e)| {
                        if existing_panel_set.contains(&i) {
                            None
                        } else {
                            Some(ViewLoadBundle {
                                linear_client: self.linear_client.clone(),

                                item_filter: e.view,
                                table_data: e.issue_table_data.clone(),
                                cursor: e.view_cursor.clone(),
                                loading: e.loading.clone(),
                            })
                        }
                    })
                    .collect();

                drop(view_panel_list_lock);

                let _t1 = tokio::spawn(async move {
                    // Load all DashboardViewPanels



                    let tasks: Vec<_> = view_load_bundles
                        .into_iter()
                        .map(|item| {
                            info!("Spawning Get View Panel Issues Task");

                            let cursor_handle = item.cursor.lock().unwrap();
                            let cursor = cursor_handle.clone();
                            drop(cursor_handle);

                            // Set ViewPanel loading state to true
                            item.loading.store(true, Ordering::Relaxed);

                            tokio::spawn(async move {
                                let linear_client_lock = item.linear_client.lock().await;
                                let client = linear_client_lock.as_ref().unwrap();

                                let res = client
                                    .issues(serde_json::from_value(serde_json::to_value(item.item_filter.filter_data).unwrap()).unwrap(), 
                                        cursor
                                    ).await;

                                let mut view_panel_data_lock = item.table_data.lock().unwrap();
                                let mut cursor_handle = item.cursor.lock().unwrap();

                                debug!("client.issues() - Returned: {:?}", res);

                                if let Ok(Some(x)) = res {

                                    let issues: Vec<Issue> = x.issues.nodes;

                                    *view_panel_data_lock = issues;

                                    *cursor_handle = Some(GraphQLCursor{
                                        platform: Platform::Linear,
                                        has_next_page: x.issues.page_info.has_next_page,
                                        end_cursor: x.issues.page_info.end_cursor
                                    });

                                    item.loading.store(false, Ordering::Relaxed);
                                }
                                debug!(
                                    "New dashboard_view_panel.issue_table_data: {:?}",
                                    view_panel_data_lock
                                );
                            })
                        })
                        .collect();

                    // await the tasks for resolve's to complete and give back our items
                    let mut items = vec![];
                    for task in tasks {
                        items.push(task.await.expect("LoadViewIssues failed"));
                    }
                    // verify that we've got the results
                    for item in &items {
                        info!("LoadViewIssues Result: {:?}", item);
                    }
                });
            }
            AppEvent::PaginateDashboardView => {
                let view_panel_list_handle = self.linear_dashboard_view_panel_list.lock().unwrap();

                let is_loading = &view_panel_list_handle[self.view_panel_to_paginate].loading;

                // If already loading something, don't try again
                if is_loading.load(Ordering::Relaxed) {
                    return;
                }

                // Set ViewPanel loading state to true
                is_loading.store(true, Ordering::Relaxed);

                let linear_client_handle = self.linear_client.clone();

                // let linear_config_lock = self.linear_client.config.lock().unwrap();
                // let linear_config = linear_config_lock.clone();
                // drop(linear_config_lock);

                let view_panel_view_obj = view_panel_list_handle[self.view_panel_to_paginate]
                    .view
                    .clone();

                let cursor_lock = view_panel_list_handle[self.view_panel_to_paginate]
                    .view_cursor
                    .lock()
                    .unwrap();
                let cursor = cursor_lock.clone();

                let view_panel_issue_handle = view_panel_list_handle[self.view_panel_to_paginate]
                    .issue_table_data
                    .clone();
                let cursor_handle = view_panel_list_handle[self.view_panel_to_paginate]
                    .view_cursor
                    .clone();

                let loading_handle = view_panel_list_handle[self.view_panel_to_paginate]
                    .loading
                    .clone();

                drop(cursor_lock);
                drop(view_panel_list_handle);

                let _t1 = tokio::spawn(async move {
                    let res = if let Some(linear_client) = &*linear_client_handle.lock().await {
                        linear_client.issues(serde_json::from_value(serde_json::to_value(view_panel_view_obj.filter_data).unwrap()).unwrap(), cursor).await
                    } else {
                        return;
                    };

                    let mut view_panel_data_lock = view_panel_issue_handle.lock().unwrap();
                    let mut cursor = cursor_handle.lock().unwrap();

                    let mut current_view_issues = view_panel_data_lock.clone();

                    if let Ok(Some(x)) = res {
                        let mut issues: Vec<Issue> = x.issues.nodes;
                        current_view_issues.append(&mut issues);
                        *view_panel_data_lock = current_view_issues.clone();
                        *cursor = Some(GraphQLCursor{
                            platform: Platform::Linear,
                            has_next_page: x.issues.page_info.has_next_page,
                            end_cursor: x.issues.page_info.end_cursor
                        });
                        loading_handle.store(false, Ordering::Relaxed);
                    }
                    info!(
                        "New dashboard_view_panel.issue_table_data: {:?}",
                        view_panel_data_lock
                    );
                });
            }
            AppEvent::LoadIssueOpData => {
                let op_interface_loading_handle = self.linear_issue_op_interface.loading.clone();

                // If already loading something, don't try again
                if op_interface_loading_handle.load(Ordering::Relaxed) {
                    return;
                }

                let current_op: IssueModificationOp =
                    match self.linear_issue_op_interface.current_op {
                        Some(x) => x,
                        None => return,
                    };

                // If current_op is ModifyTitle, return since no data needs to be loaded
                if current_op == IssueModificationOp::Title {
                    return;
                }

                // Set Loading 'true' before fetch
                op_interface_loading_handle.store(true, Ordering::Relaxed);

                let issue_op_data_handle = self.linear_issue_op_interface.obj_data.clone();

                let linear_client_handle = self.linear_client.clone();

                // Check that an Issue is selected, if not return
                let selected_issue = match fetch_selected_view_panel_issue(self) {
                    Some(x) => x,
                    None => return,
                };

                // Get the Issue's team,
                let selected_team = selected_issue.team.id;

                // Get Cursor
                let issue_op_cursor_lock = self.linear_issue_op_interface.cursor.lock().unwrap();
                let issue_op_cursor: GraphQLCursor = issue_op_cursor_lock.clone();
                drop(issue_op_cursor_lock);

                let issue_op_cursor_handle = self.linear_issue_op_interface.cursor.clone();

                let _t1 = tokio::spawn(async move {

                    let res = if let Some(client) = &*linear_client_handle.lock().await {
                        match current_op {
                            IssueModificationOp::Cycle => {
                                IssueFieldResponse::Cycles(client.team_cycles(&selected_team, Some(issue_op_cursor)).await)
                            },
                            IssueModificationOp::Project => {
                                IssueFieldResponse::Projects(client.team_projects(&selected_team, Some(issue_op_cursor)).await)
                            },
                            IssueModificationOp::Assignee => {
                                IssueFieldResponse::TeamMembers(client.team_members(&selected_team, Some(issue_op_cursor)).await)
                            },
                            IssueModificationOp::WorkflowState => {
                                IssueFieldResponse::States(client.team_states(&selected_team, Some(issue_op_cursor)).await)
                            }
                            _ => {panic!("Unsupported op!")}
                        }
                    } else {
                        return;
                    };

                    let mut issue_op_cursor_data_lock = issue_op_cursor_handle.lock().unwrap();
                    op_interface_loading_handle.store(false, Ordering::Relaxed);

                    let mut issue_op_data_lock = issue_op_data_handle.lock().unwrap();

                    match res {
                        IssueFieldResponse::Cycles(Ok(Some(cycles_resp))) => {
                            issue_op_data_lock.cycles.append(
                                &mut cycles_resp
                                    .cycles
                                    .nodes
                                    .into_iter()
                                    //.map(|e| IssueFieldObject::Cycle(e))
                                    .collect(),
                            );
                            *issue_op_cursor_data_lock = GraphQLCursor{
                                platform: Platform::Linear,
                                has_next_page: cycles_resp.cycles.page_info.has_next_page,
                                end_cursor: cycles_resp.cycles.page_info.end_cursor
                            }
                        }
                        IssueFieldResponse::Projects(Ok(Some(projects_resp))) => {
                            issue_op_data_lock.projects.append(
                                &mut projects_resp
                                    .team
                                    .projects
                                    .nodes
                                    .into_iter()
                                    //.map(|e| IssueFieldObject::Project(e))
                                    .collect(),
                            );
                            *issue_op_cursor_data_lock = GraphQLCursor{
                                platform: Platform::Linear,
                                has_next_page: projects_resp.team.projects.page_info.has_next_page,
                                end_cursor: projects_resp.team.projects.page_info.end_cursor
                            }
                        }
                        IssueFieldResponse::TeamMembers(Ok(Some(members_resp))) => {
                            issue_op_data_lock.users.append(
                                &mut members_resp
                                    .team
                                    .members
                                    .nodes
                                    .into_iter()
                                    //.map(|e| IssueFieldObject::TeamMember(e))
                                    .collect(),
                            );
                            *issue_op_cursor_data_lock = GraphQLCursor{
                                platform: Platform::Linear,
                                has_next_page: members_resp.team.members.page_info.has_next_page,
                                end_cursor: members_resp.team.members.page_info.end_cursor
                            }
                        }
                        IssueFieldResponse::States(Ok(Some(states_resp))) => {
                            issue_op_data_lock.workflow_states.append(
                                &mut states_resp
                                    .workflow_states
                                    .nodes
                                    .into_iter()
                                    //.map(|e| IssueFieldObject::State(e))
                                    .collect(),
                            );
                            *issue_op_cursor_data_lock = GraphQLCursor{
                                platform: Platform::Linear,
                                has_next_page: states_resp.workflow_states.page_info.has_next_page,
                                end_cursor: states_resp.workflow_states.page_info.end_cursor
                            }
                        }
                        _ => {
                            // TODO: Improve message
                            error!("IssueFieldResponse Error");
                            panic!("IssueFieldResponse Error");
                        }
                    }
                });
            }
            AppEvent::UpdateIssue => {
                let selected_value_id: String;
                let issue_obj_opt: Option<IssueFieldObject> = fetch_selected_value(self);

                let issue_id: String;
                let selected_issue_opt = fetch_selected_view_panel_issue(self);
                let issue_obj = if let Some(x) = selected_issue_opt { x } else { return; };
                issue_id = issue_obj.id;

                let current_op: IssueModificationOp =
                    match self.linear_issue_op_interface.current_op {
                        Some(op) => op,
                        None => return,
                    };

                let mut issue_update = IssueUpdateInput {
                    title: None,
                    description: None,
                    description_data: None,
                    assignee_id: None,
                    parent_id: None,
                    priority: None,
                    estimate: None,
                    subscriber_ids: None,
                    label_ids: None,
                    team_id: None,
                    cycle_id: None,
                    project_id: None,
                    project_milestone_id: None,
                    state_id: None,
                    board_order: None,
                    sort_order: None,
                    sub_issue_sort_order: None,
                    due_date: None,
                    trashed: None,
                    sla_breaches_at: None,
                    snoozed_until_at: None,
                    snoozed_by_id: None,
                };

                // TODO: Prevent sending update query if nothing selected
                match current_op {
                    IssueModificationOp::Title => {
                        issue_update.title = Some(self.title_entry.input.input.clone());
                        selected_value_id = self.title_entry.input.input.clone();
                    }
                    _ => match fetch_selected_value(self) {
                        Some(obj) => match obj {
                            IssueFieldObject::State(state) => {
                                issue_update.state_id = Some(state.id.clone());
                                selected_value_id = state.id.clone();
                            }
                            IssueFieldObject::TeamMember(assignee) => {
                                issue_update.assignee_id = Some(assignee.id.clone());
                                selected_value_id = assignee.id.clone();
                            }
                            IssueFieldObject::Project(project) => {
                                issue_update.project_id = Some(project.id.clone());
                                selected_value_id = project.id.clone();
                            }
                            IssueFieldObject::Cycle(cycle) => {
                                issue_update.cycle_id = Some(cycle.id.clone());
                                selected_value_id = cycle.id.clone();
                            }
                        },
                        _ => {
                            return;
                        }
                    },
                }

                let linear_client_handle = self.linear_client.clone();

                let view_panel_list_arc = self.linear_dashboard_view_panel_list.clone();

                // Spawn task to issue command to update issue
                let _t3 = tokio::spawn(async move {
                    let res = if let Some(client) = &*linear_client_handle.lock().await {
                        client.update_issue(&issue_id, issue_update).await
                    } else {
                        return;
                    };

                    // then can match linear_issue_display.issue_table_data using selected_issue["id"]
                    // and update linear_issue_display.issue_table_data[x]["state"] with selected_workflow_state

                    if let Ok(Some(_query_response)) = res {
                        // If update succeeded, iterate over all Issues in all ViewPanels
                        // and set issue["state" | "assignee" | ...] = state_obj
                        //     where id matches 'issue_id'

                        let view_panel_list_handle = view_panel_list_arc.lock().unwrap();
                        for view_panel in view_panel_list_handle.iter() {
                            // Iterate over ViewPanel Issues
                            let mut issue_list_handle = view_panel.issue_table_data.lock().unwrap();

                            for issue_obj in issue_list_handle.iter_mut() {
                                if issue_obj.id == issue_id {
                                    match current_op {
                                        IssueModificationOp::Title => {
                                            issue_obj.title = selected_value_id.clone()
                                        }
                                        _ => {
                                            if let Some(issue_field_obj) = &issue_obj_opt {
                                                match issue_field_obj {
                                                    IssueFieldObject::State(state) => {
                                                        issue_obj.state = serde_json::from_value(serde_json::to_value(state.clone()).unwrap()).unwrap();
                                                    }
                                                    IssueFieldObject::TeamMember(assignee) => {
                                                        issue_obj.assignee = serde_json::from_value(serde_json::to_value(assignee.clone()).unwrap()).unwrap();
                                                    }
                                                    IssueFieldObject::Project(project) => {
                                                        issue_obj.project = serde_json::from_value(serde_json::to_value(project.clone()).unwrap()).unwrap();
                                                    }
                                                    IssueFieldObject::Cycle(cycle) => {
                                                        issue_obj.cycle = serde_json::from_value(serde_json::to_value(cycle.clone()).unwrap()).unwrap();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }
        }
    }
}
