use tui::{
    style::{Color},
};

// View Panel Colors
pub const SELECTED_COMPONENT_BORDER: Color = Color::Yellow;
pub const API_REQ_NUM: Color = Color::Rgb(173u8, 252u8, 3u8);


// Command Bar Colors

// Green
pub const ADD_VIEW_CMD_ACTIVE: Color = Color::Rgb( 52u8, 227u8, 28u8 );
pub const ADD_VIEW_CMD_INACTIVE: Color = Color::Rgb( 39u8, 170u8, 21u8 );

// Purple
pub const REPLACE_VIEW_CMD_ACTIVE: Color = Color::Rgb( 107u8, 83u8, 250u8 );
pub const REPLACE_VIEW_CMD_INACTIVE: Color = Color::Rgb( 40u8, 7u8, 242u8 );

// Red
pub const REMOVE_VIEW_CMD_ACTIVE: Color = Color::Rgb( 184u8, 4u8, 4u8 );
pub const REMOVE_VIEW_CMD_INACTIVE: Color = Color::Rgb( 138u8, 3u8, 3u8 );

