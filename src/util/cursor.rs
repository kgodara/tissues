use crate::app::{ Platform };

#[derive(Debug)]
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

    pub fn linear_cursor_from_page_info(page_info: serde_json::Value) -> Option<GraphQLCursor> {

        let mut return_option = None;

        match page_info {
            serde_json::Value::Object(page_object) => {
                match &page_object["hasNextPage"] {
                    serde_json::Value::String(page_bool) => {
                        info!("hasNextPage was String");
                        /*
                        match &page_object["endCursor"] {
                            serde_json::Value::String(cursor_str) => {
                                
                                return_option = Some(GraphQLCursor {
                                    platform: Platform::Linear,
                                    has_next_page: *page_bool.parse::<bool>().ok().get_or_insert(false),
                                    end_cursor: cursor_str.clone(),
                                });

                            },
                            _ => {},
                        }
                        */
                    },
                    serde_json::Value::Bool(page_bool) => {
                        info!("hasNextPage was bool");

                        match &page_object["endCursor"] {
                            serde_json::Value::String(cursor_str) => {
                                
                                return_option = Some(GraphQLCursor {
                                    platform: Platform::Linear,
                                    has_next_page: *page_bool,
                                    end_cursor: cursor_str.clone(),
                                });

                            },
                            _ => {},
                        }

                    },
                    _ => {},
                }
            },
            _ => {},
        }

        return return_option;
            
    }
}