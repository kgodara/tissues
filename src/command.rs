
use tokio::sync::oneshot;
use crate::util::StatefulList;

use crate::util::GraphQLCursor;

use crate::linear::LinearConfig as LinearConfig;

use std::sync::{ Arc, Mutex };

#[derive(Debug)]
pub enum Command {
    LoadLinearTeams {
        api_key: Option<String>,
        resp: Responder<serde_json::Value>
    },
    LoadLinearIssues {
        // api_key: Option<String>,
        linear_config: LinearConfig,
        selected_team: serde_json::Value,
        resp: Responder<serde_json::Value>
    },
    LoadWorkflowStates {
        api_key: Option<String>,
        selected_team: serde_json::Value,
        resp: Responder<serde_json::Value>,
    },
    UpdateIssueWorkflowState {
        api_key: Option<String>,
        selected_issue: serde_json::Value,
        selected_workflow_state: serde_json::Value,
        resp: Responder<serde_json::Value>,
    },
}

type Responder<T> = oneshot::Sender<Option<T>>;
