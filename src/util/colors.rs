use tui::{
    style::{Color},
};

// Component Colors
pub const CUSTOM_VIEW_SELECT_TABLE_TITLE: Color = Color::Rgb(255u8, 255u8, 255u8);
pub const DASHBOARD_VIEW_LIST_TABLE_TITLE: Color = Color::Rgb(255u8, 255u8, 255u8);

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
pub const DELETE_VIEW_CMD_ACTIVE: Color = Color::Rgb( 184u8, 4u8, 4u8 );
pub const DELETE_VIEW_CMD_INACTIVE: Color = Color::Rgb( 138u8, 3u8, 3u8 );

// Teal (25% darker for inactive) -- https://pinetools.com/darken-color
pub const REFRESH_PANEL_CMD_ACTIVE: Color = Color::Rgb( 81u8, 193u8, 177u8 );
pub const REFRESH_PANEL_CMD_INACTIVE: Color = Color::Rgb( 53u8, 151u8, 137u8 );

// Orange (25% darker for inactive) -- https://pinetools.com/darken-color
pub const MODIFY_WORKFLOW_STATE_CMD_ACTIVE: Color = Color::Rgb( 252u8, 132u8, 4u8 );
pub const MODIFY_WORKFLOW_STATE_CMD_INACTIVE: Color = Color::Rgb( 189u8, 99u8, 2u8 );

