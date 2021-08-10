use tui::{
    layout::{Constraint},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::util::ui::style_color_from_hex_str;

use serde_json::Value;

struct CommandValue {
    key_char: char,
    label: str,
}

// Describe route-specific commands
enum CommandBarType {
    Dashboard,
    ViewList
}

enum DashboardCommand {
    RefreshPanel = CommandValue { key_char: 'r', label: "Refresh" },
    ModifyWorkflowState = CommandValue { key_char: 'm', label: "Modify Workflow State" },
}

enum ViewListCommand {
    RemoveView = CommandValue { key_char: 'r', label: "Remove View" },
}

enum Command {
    Dashboard(DashboardCommand),
    ViewList(ViewListCommand),
}


pub struct CommandBar {
    pub command_bar_type: CommandBarType,
    
    // Dashboard Command States
    refresh_panel_active: bool,
    modify_workflow_state_active: bool,

    // View List Command States
    remove_view_active: bool,
}

impl CommandBar {

    fn command_bar_with_type(cmd_bar_type: CommandBarType) -> CommandBar {
        CommandBar {
            command_bar_type: CommandBarType
        }
    }

    // Dashboard Command Setters
    fn set_refresh_panel_active(&self, state: bool) {
        match self.command_bar_type {
            CommandBarType::Dashboard => {
                self.refresh_panel_active = state;
            },
            _ => {
                error!("'set_refresh_panel_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
                panic!("'set_refresh_panel_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
            },
        }
    }

    fn set_modify_workflow_state_active(&self, state: bool) {
        match self.command_bar_type {
            CommandBarType::Dashboard => {
                self.modify_workflow_state_active = state;
            },
            _ => {
                error!("'set_modify_workflow_state_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
                panic!("'set_modify_workflow_state_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
            },
        }
    }


    // View List Command Setters
    fn set_remove_view_active(&self, state: bool) {
        match self.command_bar_type {
            CommandBarType::ViewList => {
                self.remove_view_active = state;
            },
            _ => {
                error!("'set_remove_view_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
                panic!("'set_remove_view_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
            },
        }
    }

}