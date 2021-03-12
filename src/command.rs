
use tokio::sync::oneshot;
use crate::util::StatefulList;

use std::sync::{ Arc, Mutex };

#[derive(Debug)]
pub enum Command {
    LoadLinearTeams {
        api_key: Option<String>,
        resp: TeamResponder<StatefulList<serde_json::Value>>
    },
    LoadLinearIssues {
        api_key: Option<String>,
        selected_team: serde_json::Value,
        resp: IssueResponder<serde_json::Value>
    }
}


// type TeamResponder<T> = oneshot::Sender<Arc<Mutex<Option<T>>>>;
type TeamResponder<T> = oneshot::Sender<Option<T>>;
type IssueResponder<T> = oneshot::Sender<Option<T>>;