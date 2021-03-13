
use tokio::sync::oneshot;
use crate::util::StatefulList;

use std::sync::{ Arc, Mutex };

#[derive(Debug)]
pub enum Command {
    LoadLinearTeams {
        api_key: Option<String>,
        resp: Responder<serde_json::Value>
    },
    LoadLinearIssues {
        api_key: Option<String>,
        selected_team: serde_json::Value,
        resp: Responder<serde_json::Value>
    }
}

type Responder<T> = oneshot::Sender<Option<T>>;
