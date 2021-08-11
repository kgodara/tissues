use tui::{
    style::{Color}
};

use crate::util::colors;

#[derive(Debug, Clone)]
pub enum DashboardCommand {
    RefreshPanel,
    ModifyWorkflowState,
}

#[derive(Debug, Clone)]
pub enum ViewListCommand {
    RemoveView,
}

#[derive(Debug, Clone)]
pub enum Command {
    Dashboard(DashboardCommand),
    ViewList(ViewListCommand),
}

pub struct CommandValue<'a> {
    pub key_char: char,
    pub cmd_type: Command,
    pub label: &'a str,
    pub active_color: Color,
    pub inactive_color: Color,
}

pub struct CommandList<'a> {
    pub dashboard: Vec<CommandValue<'a>>,
    pub view_list: Vec<CommandValue<'a>>,
}



impl Default for CommandList<'_> {
    fn default() -> CommandList<'static> {
        CommandList {
            dashboard: vec![ 
                CommandValue { key_char: 'r',
                    cmd_type: Command::Dashboard(DashboardCommand::RefreshPanel),
                    label: "Refresh",
                    active_color: colors::REFRESH_PANEL_CMD_ACTIVE,
                    inactive_color: colors::REFRESH_PANEL_CMD_INACTIVE
                },
                CommandValue { key_char: 'm',
                    cmd_type: Command::Dashboard(DashboardCommand::ModifyWorkflowState),
                    label: "Modify Workflow State",
                    active_color: colors::MODIFY_WORKFLOW_STATE_CMD_ACTIVE,
                    inactive_color: colors::MODIFY_WORKFLOW_STATE_CMD_INACTIVE
                },
            ],
            view_list: vec![
                CommandValue { key_char: 'r',
                    cmd_type: Command::ViewList(ViewListCommand::RemoveView),
                    label: "Remove View",
                    active_color: colors::REMOVE_VIEW_CMD_ACTIVE,
                    inactive_color: colors::REMOVE_VIEW_CMD_INACTIVE,
                },
            ],
        }
    }
}