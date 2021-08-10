
use tokio::sync::oneshot;

use serde_json::Value;


use crate::linear::LinearConfig as LinearConfig;
use crate::linear::view_resolver::ViewLoader;

use crate::util::GraphQLCursor as GraphQLCursor;

use std::collections::HashMap;

use std::sync::{ Arc, Mutex };

#[derive(Debug)]
pub enum IOEvent {
    LoadLinearTeamTimeZones {
        linear_config: LinearConfig,
        resp: oneshot::Sender<Vec<(String, String)>>,
    },
    LoadCustomViews {
        linear_config: LinearConfig,
        linear_cursor: GraphQLCursor,
        resp: Responder<Value>
    },
    LoadViewIssues {
        linear_config: LinearConfig,
        view: Value,
        team_tz_lookup: HashMap<String, String>,
        tz_offset_lookup: Arc<Mutex<HashMap<String, f64>>>,
        issue_data: Arc<Mutex<Vec<Value>>>,
        view_loader: Option<ViewLoader>,
        resp: oneshot::Sender<(Vec<Value>, ViewLoader, u32)>,
    },
    LoadWorkflowStates {
        linear_config: LinearConfig,
        team: Value,
        resp: Responder<Value>,
    },
    UpdateIssueWorkflowState {
        linear_config: LinearConfig,
        issue_id: String,
        workflow_state_id: String,
        resp: Responder<Value>,
    },
    LoadLinearTeams {
        api_key: Option<String>,
        resp: Responder<Value>
    },
}

type Responder<T> = oneshot::Sender<Option<T>>;
