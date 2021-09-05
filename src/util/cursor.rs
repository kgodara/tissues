use crate::app::{ Platform };

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct GraphQLCursor {
    pub platform: Platform,
    pub has_next_page: bool,
    pub end_cursor: String,
}

impl Default for GraphQLCursor {
    fn default() -> GraphQLCursor {
        GraphQLCursor {
            platform: Platform::Na,
            has_next_page: false,
            end_cursor: String::default()
        }
    }
}

impl GraphQLCursor {
    pub fn platform_cursor(cursor_platform: Platform) -> GraphQLCursor {
        GraphQLCursor {
            platform: cursor_platform,
            has_next_page: false,
            end_cursor: String::default(),
        }
    }

    pub fn linear_cursor_from_page_info(page_info: Value) -> Option<GraphQLCursor> {

        let mut return_option = None;

        if let Value::Object(page_object) = page_info {
            if let Value::Bool(page_bool) = &page_object["hasNextPage"] {
                if let Value::String(cursor_str) = &page_object["endCursor"] {
                    return_option = Some(GraphQLCursor {
                        platform: Platform::Linear,
                        has_next_page: *page_bool,
                        end_cursor: cursor_str.clone(),
                    });
                } else if !page_bool {
                    // allow for null endCursor if no more pages remain
                    // https://github.com/kgodara/rust-cli/issues/78
                    if let Value::Null = &page_object["endCursor"] {
                        return_option = Some(GraphQLCursor {
                            platform: Platform::Linear,
                            has_next_page: *page_bool,
                            end_cursor: "".to_string(),
                        });
                    }
                }
            }
        }
        return_option
    }
}