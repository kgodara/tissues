use super::config::LinearConfig;
use super::query::get_teams as exec_get_teams;
use super::query::get_issues_by_team as exec_get_issues_by_team;

use super::error::LinearClientError;

use crate::errors::*;

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

    pub fn get_teams(&self) -> Result<serde_json::Value, LinearClientError> {

        let linear_api_key;

        info!("self.config.api_key: {:?}", self.config.api_key);

        match &self.config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };

        info!("linear_api_key: {:?}", linear_api_key);


        let query_response = exec_get_teams(linear_api_key)?;

        let ref team_nodes = query_response["data"]["teams"]["nodes"];

        Ok(team_nodes.clone())
    }

    pub fn get_issues_by_team(&self, variables: serde_json::Map<String, serde_json::Value>) -> Result<serde_json::Value, LinearClientError> {

        info!("Calling exec_get_issues_by_team - variables: {:?}", variables);

        let linear_api_key;
        match &self.config.api_key {
            Some(x) => linear_api_key = x,
            None => return Err(LinearClientError::InvalidConfig(ConfigError::CredentialsNotFound{ platform: String::from("Linear") })),
        };


        let query_response = exec_get_issues_by_team(linear_api_key, variables)?;

        let ref issue_nodes = query_response["data"]["team"]["issues"]["nodes"];

        Ok(issue_nodes.clone())
    }

}