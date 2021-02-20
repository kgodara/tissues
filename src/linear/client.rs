use super::config::LinearConfig;
use super::query::get_linear_teams;
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

        let query_response = get_linear_teams(&self.config.api_key)?;

        let ref team_nodes = query_response["data"]["teams"]["nodes"];

        Ok(team_nodes.clone())

    }

}