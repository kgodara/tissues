

#[derive(Debug)]
pub enum Command {
    LoadLinearTeams {
        api_key: Option<String>,
    },
    LoadLinearIssues {
        selected_team: serde_json::Value,
    }
}