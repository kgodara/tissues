
use tokio::sync::oneshot;
use crate::util::StatefulList;

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
        issue_data: Arc<Mutex<Option<Value>>>,
        view_loader: Option<ViewLoader>,
        resp: oneshot::Sender<(Vec<Value>, ViewLoader, u32)>,
    },
    LoadLinearTeams {
        api_key: Option<String>,
        resp: Responder<Value>
    },
    LoadLinearIssues {
        linear_config: LinearConfig,
        selected_team: Value,
        resp: Responder<Value>
    },
    LoadLinearIssuesPaginate {
        linear_config: LinearConfig,
        linear_cursor: GraphQLCursor,
        selected_team: Value,
        resp: Responder<Value>
    },
    LoadWorkflowStates {
        api_key: Option<String>,
        selected_team: Value,
        resp: Responder<Value>,
    },
    UpdateIssueWorkflowState {
        api_key: Option<String>,
        selected_issue: Value,
        selected_workflow_state: Value,
        resp: Responder<Value>,
    },
}

type Responder<T> = oneshot::Sender<Option<T>>;
