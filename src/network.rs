
use tokio::sync::oneshot;

use serde_json::Value;


use crate::linear::{
    LinearConfig as LinearConfig,
    schema::{view_query, CustomView},
};

use crate::constants::{
    IssueModificationOp
};

use crate::util::GraphQLCursor as GraphQLCursor;

#[derive(Debug)]
pub enum IOEvent {
    LoadLinearTeamTimeZones {
        linear_config: LinearConfig,
        resp: oneshot::Sender<Vec<(String, String)>>,
    },
    LoadCustomViews {
        linear_config: LinearConfig,
        linear_cursor: GraphQLCursor,
        resp: oneshot::Sender<anyhow::Result<view_query::ResponseData>>
    },
    LoadViewer {
        api_key: String,
        resp: Responder<Value>,
    },
    LoadViewIssues {
        linear_config: LinearConfig,
        view: CustomView,
        view_cursor: Option<GraphQLCursor>,
        resp: oneshot::Sender<(Vec<Value>, GraphQLCursor, u32)>,
    },

    LoadOpData {
        op: IssueModificationOp,
        linear_config: LinearConfig,
        linear_cursor: GraphQLCursor,
        team_id: String,
        resp: Responder<Value>,
    },
    UpdateIssue {
        op: IssueModificationOp,
        linear_config: LinearConfig,
        issue_id: String,
        ref_id: String,
        resp: Responder<Value>,
    }
}

type Responder<T> = oneshot::Sender<Option<T>>;
