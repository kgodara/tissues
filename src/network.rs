
use tokio::sync::oneshot;
use crate::util::StatefulList;


use crate::linear::LinearConfig as LinearConfig;
use crate::util::GraphQLCursor as GraphQLCursor;

use std::sync::{ Arc, Mutex };

#[derive(Debug)]
pub enum IOEvent {
    LoadLinearTeams {
        api_key: Option<String>,
        resp: Responder<serde_json::Value>
    },
    LoadLinearIssues {
        linear_config: LinearConfig,
        selected_team: serde_json::Value,
        resp: Responder<serde_json::Value>
    },
    LoadLinearIssuesPaginate {
        linear_config: LinearConfig,
        linear_cursor: GraphQLCursor,
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
