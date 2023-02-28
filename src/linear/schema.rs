use graphql_client::GraphQLQuery;
use serde_json::{ Map, Value };

// https://github.com/graphql-rust/graphql-client#custom-scalars
pub type JSONObject = Map<String, Value>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/linear/linear_schema.json",
    query_path = "queries/linear/fetch_custom_views.graphql",
    response_derives = "Debug,Clone,Serialize,Default"
)]
pub struct ViewQuery;

pub type CustomView = view_query::ViewQueryCustomViewsNodes;
pub type CustomViewResponseData = view_query::ResponseData;