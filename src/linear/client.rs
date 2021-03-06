use super::config::LinearConfig;
use super::query::get_teams as exec_get_teams;
use super::query::get_issues_by_team as exec_get_issues_by_team;

use crate::graphql::GraphQLError;

pub struct LinearClient {
    config: LinearConfig,
}

impl Default for LinearClient {
    fn default() -> LinearClient {
        LinearClient { config: LinearConfig::default() }
    }
}

impl LinearClient {

    fn set_config(&mut self, new_config: LinearConfig) {
        self.config = new_config;
    }

    pub fn get_teams(&self) -> Result<serde_json::Value, GraphQLError> {

        let query_response = exec_get_teams(&self.config.api_key)?;

        let ref team_nodes = query_response["data"]["teams"]["nodes"];

        Ok(team_nodes.clone())

    }

    pub fn get_issues_by_team(&self, variables: serde_json::Map<String, serde_json::Value>) -> Result<serde_json::Value, GraphQLError> {

        info!("Calling exec_get_issues_by_team - variables: {:?}", variables);

        let query_response = exec_get_issues_by_team(&self.config.api_key, variables)?;

        let ref issue_nodes = query_response["data"]["team"]["issues"]["nodes"];

        Ok(issue_nodes.clone())
    }

}