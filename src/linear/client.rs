use super::config::{ LinearConfig };

use anyhow::{anyhow,Result};
use std::{boxed::Box};

use crate::{
    util::{ GraphQLCursor },
};

use reqwest::header;
use graphql_client::reqwest::post_graphql;

use super::schema::{

    // Custom Views
    ViewQuery, CustomViewVariables, CustomViewResponseData,
    // Viewer
    ViewerQuery, ViewerVariables, ViewerResponseData,

    // Cycles
    cycles_query,
    CyclesQuery, CyclesVariables, CyclesResponseData, Cycle,

    // Projects By Team
    TeamProjectsQuery, ProjectsVariables, ProjectsResponseData, Project,

    // Users By Team (Members)
    TeamMembersQuery, TeamMembersVariables, TeamMembersResponseData, TeamMember,

    // Workflow States
    states_query,
    StatesQuery, StatesVariables, StatesResponseData, State,

    // Update Issue
    IssueUpdateMut, IssueUpdateInput, IssueUpdateVariables, IssueUpdateResponseData,

    // Issues
    IssuesQuery, IssuesVariables, IssueFilter, IssuesResponseData,
};

pub enum IssueFieldResponse {
    Cycles(Result<Option<CyclesResponseData>>),
    Projects(Result<Option<ProjectsResponseData>>),
    TeamMembers(Result<Option<TeamMembersResponseData>>),
    States(Result<Option<StatesResponseData>>),
}

#[derive(Debug, Clone)]
pub enum IssueFieldObject {
    Cycle(Cycle),
    Project(Project),
    TeamMember(TeamMember),
    State(State),
}


pub struct LinearClient {
    pub client: reqwest::Client,
    pub config: LinearConfig,
}

impl LinearClient {

    pub fn with_config(config: LinearConfig) -> Result<LinearClient> {

        let linear_api_key = match &config.api_key {
            Some(x) => x.to_string(),
            None => return Err(anyhow!("LinearConfig missing API key")),
        };

        let mut headers = header::HeaderMap::new();

        let mut auth_value = header::HeaderValue::from_str(&format!("{}", linear_api_key))?;
        auth_value.set_sensitive(true);
    
        headers.insert(header::AUTHORIZATION, auth_value);
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
        

        Ok(LinearClient {
            client: reqwest::Client::builder()
                .user_agent("tissues")
                .default_headers(headers.clone())
                .build()
                .unwrap(),
            config: config
        })
    }

    pub async fn custom_views(&self, cursor_opt: Option<GraphQLCursor>) -> Result<Option<CustomViewResponseData>> {

        let variables = CustomViewVariables {
            first_num: Some(self.config.custom_view_page_size as i64),
            after_cursor: if let Some(cursor) = cursor_opt { cursor.end_cursor } else { None },
        };
        
        debug!("custom_views() - Variables: {:?}", variables);

        Ok(
            post_graphql::<ViewQuery, _>(&self.client, "https://api.linear.app/graphql", variables).await?.data
        )
    }

    pub async fn viewer(&self) -> Result<Option<ViewerResponseData>> {

        let variables = ViewerVariables{};
        Ok(
            post_graphql::<ViewerQuery, _>(&self.client, "https://api.linear.app/graphql", variables).await?.data
        )
    }

    pub async fn issues(&self, filter: IssueFilter, cursor_opt: Option<GraphQLCursor>) -> Result<Option<IssuesResponseData>> {
        let variables = IssuesVariables {
            first_num: Some(self.config.view_panel_page_size as i64),
            after_cursor: if let Some(cursor) = cursor_opt { cursor.end_cursor } else { None },
            filter: filter,
        };

        debug!("issues() - Variables: {:?}", variables);

        Ok(
            post_graphql::<IssuesQuery, _>(&self.client, "https://api.linear.app/graphql", variables).await?.data
        )
    }

    pub async fn team_cycles(&self, team_id: &str, cursor: Option<GraphQLCursor>) -> Result<Option<CyclesResponseData>> {

        type CycleFilter = cycles_query::CycleFilter;
        type TeamFilter = cycles_query::TeamFilter;
        type IDComparator = cycles_query::IDComparator;
    
        let filter = CycleFilter {
            id: None,
            created_at: None,
            updated_at: None,
            number: None,
            name: None,
            starts_at: None,
            ends_at: None,
            completed_at: None,
            is_active: None,
            is_next: None,
            is_previous: None,
            is_future: None,
            is_past: None,
            team: Box::new(Some(TeamFilter{
                id: Some( IDComparator {
                    eq: Some(team_id.to_string()),
                    neq: None,
                    in_: None,
                    nin: None,
                }),
                created_at: None,
                updated_at: None,
                name: None,
                key: None,
                description: None,
                issues: Box::new(None),
                and: Box::new(None),
                or: Box::new(None),
            })),
            issues: Box::new(None),
            and: None,
            or: None,
        };
    
        self.cycles(Some(filter), cursor).await
    }
    pub async fn cycles(&self, cycle_filter: Option<cycles_query::CycleFilter>, cursor_opt: Option<GraphQLCursor>) -> Result<Option<CyclesResponseData>> {

        let variables = CyclesVariables {
            first_num: Some(self.config.issue_op_page_size as i64),
            after_cursor: if let Some(cursor) = cursor_opt { cursor.end_cursor } else { None },
            cycle_filter: cycle_filter,
        };
        Ok(
            post_graphql::<CyclesQuery, _>(&self.client, "https://api.linear.app/graphql", variables).await?.data
        )
    }

    pub async fn team_projects(&self, team_id: &str, cursor_opt: Option<GraphQLCursor>) -> Result<Option<ProjectsResponseData>> {
        let variables = ProjectsVariables {
            first_num: Some(self.config.issue_op_page_size as i64),
            after_cursor: if let Some(cursor) = cursor_opt { cursor.end_cursor } else { None },
            ref_: team_id.to_string(),
        };
        
        Ok(
            post_graphql::<TeamProjectsQuery, _>(&self.client, "https://api.linear.app/graphql", variables).await?.data
        )
    }

    pub async fn team_members(&self, team_id: &str, cursor_opt: Option<GraphQLCursor>) -> Result<Option<TeamMembersResponseData>> {
        let variables = TeamMembersVariables {
            first_num: Some(self.config.issue_op_page_size as i64),
            after_cursor: if let Some(cursor) = cursor_opt { cursor.end_cursor } else { None },
            ref_: team_id.to_string(),
        };
        Ok(
            post_graphql::<TeamMembersQuery, _>(&self.client, "https://api.linear.app/graphql", variables).await?.data
        )
    }

    pub async fn team_states(&self, team_id: &str, cursor: Option<GraphQLCursor>) -> Result<Option<StatesResponseData>> {

        type StateFilter = states_query::WorkflowStateFilter;
        type TeamFilter = states_query::TeamFilter;
        type IDComparator = states_query::IDComparator;
    
        let filter = StateFilter {
            id: None,
            created_at: None,
            updated_at: None,
            name: None,
            description: None,
            position: None,
            type_: None,
            team: Box::new(Some(TeamFilter{
                id: Some( IDComparator {
                    eq: Some(team_id.to_string()),
                    neq: None,
                    in_: None,
                    nin: None,
                }),
                created_at: None,
                updated_at: None,
                name: None,
                key: None,
                description: None,
                issues: Box::new(None),
                and: Box::new(None),
                or: Box::new(None),
            })),
            issues: Box::new(None),
            and: Box::new(None),
            or: Box::new(None),
        };
    
        self.states(filter, cursor).await
    
    }
    pub async fn states(&self, state_filter: states_query::WorkflowStateFilter, cursor_opt: Option<GraphQLCursor>) -> Result<Option<StatesResponseData>> {
    
        let variables = StatesVariables {
            first_num: Some(self.config.issue_op_page_size as i64),
            after_cursor: if let Some(cursor) = cursor_opt { cursor.end_cursor } else { None },
            state_filter: Some(state_filter)
        };
        Ok(
            post_graphql::<StatesQuery, _>(&self.client, "https://api.linear.app/graphql", variables).await?.data
        )
    }

    // Note: Idempotent
    pub async fn update_issue(&self, issue_id: &str, update: IssueUpdateInput) -> Result<Option<IssueUpdateResponseData>> {

        let variables = IssueUpdateVariables {
            issue_id: issue_id.to_string(),
            update,
        };
        Ok(
            post_graphql::<IssueUpdateMut, _>(&self.client, "https://api.linear.app/graphql", variables).await?.data
        )
    }

}