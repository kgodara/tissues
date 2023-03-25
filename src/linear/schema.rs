use graphql_client::GraphQLQuery;
use serde_json::{ Map, Value };

// https://github.com/graphql-rust/graphql-client#custom-scalars
pub type JSON = String;
pub type JSONObject = Map<String, Value>;
pub type DateTime = String;
pub type TimelessDate = String;

// Get Custom Views
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/linear/linear_schema.json",
    query_path = "gql/linear/custom_views.graphql",
    response_derives = "Debug,Clone,Serialize,Default",
    variables_derives = "Debug",
    skip_serializing_none,
)]
pub struct ViewQuery;

pub type CustomViewVariables = view_query::Variables;
pub type CustomViewResponseData = view_query::ResponseData;
pub type CustomView = view_query::ViewQueryCustomViewsNodes;


// Get Viewer
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/linear/linear_schema.json",
    query_path = "gql/linear/viewer.graphql",
    response_derives = "Debug,Clone,Serialize,Default",
    variables_derives = "Debug",
    skip_serializing_none,
)]
pub struct ViewerQuery;

pub type ViewerVariables = viewer_query::Variables;
pub type ViewerResponseData = viewer_query::ResponseData;
pub type Viewer = viewer_query::ViewerQueryViewer;


// Fetch Cycles
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/linear/linear_schema.json",
    query_path = "gql/linear/cycles.graphql",
    response_derives = "Debug,Clone,Serialize,Default",
    variables_derives = "Debug",
    skip_serializing_none,
)]
pub struct CyclesQuery;

pub type CyclesVariables = cycles_query::Variables;
pub type CyclesResponseData = cycles_query::ResponseData;
pub type Cycle = cycles_query::CyclesQueryCyclesNodes;

// Projects by Team
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/linear/linear_schema.json",
    query_path = "gql/linear/team_projects.graphql",
    response_derives = "Debug,Clone,Serialize,Default",
    variables_derives = "Debug",
    skip_serializing_none,
)]
pub struct TeamProjectsQuery;

pub type ProjectsVariables = team_projects_query::Variables;
pub type ProjectsResponseData = team_projects_query::ResponseData;
pub type Project = team_projects_query::TeamProjectsQueryTeamProjectsNodes;


// Team Members
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/linear/linear_schema.json",
    query_path = "gql/linear/team_members.graphql",
    response_derives = "Debug,Clone,Serialize,Default",
    variables_derives = "Debug",
    skip_serializing_none,
)]
pub struct TeamMembersQuery;

pub type TeamMembersVariables = team_members_query::Variables;
pub type TeamMembersResponseData = team_members_query::ResponseData;
pub type TeamMember = team_members_query::TeamMembersQueryTeamMembersNodes;


// Fetch States
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/linear/linear_schema.json",
    query_path = "gql/linear/states.graphql",
    response_derives = "Debug,Clone,Serialize,Default",
    variables_derives = "Debug",
    skip_serializing_none,
)]
pub struct StatesQuery;

pub type StatesVariables = states_query::Variables;
pub type StatesResponseData = states_query::ResponseData;
pub type State = states_query::StatesQueryWorkflowStatesNodes;



#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/linear/linear_schema.json",
    query_path = "gql/linear/issue_update.graphql",
    response_derives = "Debug,Clone,Serialize,Default",
    variables_derives = "Debug",
    skip_serializing_none,
)]
pub struct IssueUpdateMut;

pub type IssueUpdateVariables = issue_update_mut::Variables;
pub type IssueUpdateResponseData = issue_update_mut::ResponseData;
pub type IssueUpdateIssue = issue_update_mut::IssueUpdateMutIssueUpdateIssue;
pub type IssueUpdateInput = issue_update_mut::IssueUpdateInput;


#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gql/linear/linear_schema.json",
    query_path = "gql/linear/issues.graphql",
    response_derives = "Debug,Clone,Serialize,Default",
    variables_derives = "Debug,Deserialize",
    skip_serializing_none,
)]
pub struct IssuesQuery;

pub type IssuesVariables = issues_query::Variables;
pub type IssuesResponseData = issues_query::ResponseData;
pub type Issue = issues_query::IssuesQueryIssuesNodes;
pub type IssueFilter = issues_query::IssueFilter;