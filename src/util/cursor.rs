use crate::app::{ Platform };

// TODO: Change end_cursor to Option<String> to match gql schema
#[derive(Debug, Clone)]
pub struct GraphQLCursor {
    pub platform: Platform,
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

impl GraphQLCursor {
    pub fn with_platform(cursor_platform: Platform) -> GraphQLCursor {
        GraphQLCursor {
            platform: cursor_platform,
            has_next_page: false,
            end_cursor: None,
        }
    }
}