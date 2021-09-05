use tui::{
    style::{Color}
};

use crate::constants::colors;

#[derive(Debug, Clone)]
pub enum DashboardCommand {
    RefreshPanel,
    ModifyWorkflowState,
    ModifyAssignee,
    ModifyProject,
    ModifyCycle,
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

impl<'a> CommandValue<'a> {
    pub fn gen_label(&self) -> String {
        format!("'{}': {}", self.key_char, self.label)
    }
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
                CommandValue { key_char: 'w',
                    cmd_type: Command::Dashboard(DashboardCommand::ModifyWorkflowState),
                    label: "Modify Workflow State",
                    active_color: colors::MODIFY_WORKFLOW_STATE_CMD_ACTIVE,
                    inactive_color: colors::MODIFY_WORKFLOW_STATE_CMD_INACTIVE
                },
                CommandValue { key_char: 'a',
                    cmd_type: Command::Dashboard(DashboardCommand::ModifyAssignee),
                    label: "Modify Assignee",
                    active_color: colors::MODIFY_ASSIGNEE_CMD_ACTIVE,
                    inactive_color: colors::MODIFY_ASSIGNEE_CMD_INACTIVE,
                },
                CommandValue { key_char: 'p',
                    cmd_type: Command::Dashboard(DashboardCommand::ModifyProject),
                    label: "Modify Project",
                    active_color: colors::MODIFY_PROJECT_CMD_ACTIVE,
                    inactive_color: colors::MODIFY_PROJECT_CMD_INACTIVE,
                },
                CommandValue { key_char: 'c',
                    cmd_type: Command::Dashboard(DashboardCommand::ModifyCycle),
                    label: "Modify Cycle",
                    active_color: colors::MODIFY_CYCLE_CMD_ACTIVE,
                    inactive_color: colors::MODIFY_CYCLE_CMD_INACTIVE,
                },


            ],
            view_list: vec![
                CommandValue { key_char: 'd',
                    cmd_type: Command::ViewList(ViewListCommand::RemoveView),
                    label: "Delete View",
                    active_color: colors::DELETE_VIEW_CMD_ACTIVE,
                    inactive_color: colors::DELETE_VIEW_CMD_INACTIVE,
                },
            ],
        }
    }
}