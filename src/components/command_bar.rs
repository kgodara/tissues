use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
};

use crate::constants::{
    colors,
    command_list::{ Command, DashboardCommand, ViewListCommand, CommandList }
};

#[derive(Debug)]
pub enum CommandBarType {
    Dashboard,
    ViewList,
}


pub struct CommandBar<'a> {
    pub command_bar_type: CommandBarType,

    command_list: CommandList<'a>,
    
    // Dashboard Command States
    refresh_panel_active: bool,
    expand_issue_active: bool,

    modify_title_active: bool,
    modify_workflow_state_active: bool,
    modify_assignee_active: bool,
    modify_project_active: bool,
    modify_cycle_active: bool,

    // View List Command States
    remove_view_active: bool,
}

impl<'a> CommandBar<'a> {

    pub fn with_type(cmd_bar_type: CommandBarType) -> CommandBar<'a> {
        CommandBar {
            command_bar_type: cmd_bar_type,

            command_list: CommandList::default(),
            
            // Dashboard Command States
            refresh_panel_active: false,
            expand_issue_active: false,

            modify_title_active: false,
            modify_workflow_state_active: false,
            modify_assignee_active: false,
            modify_project_active: false,
            modify_cycle_active: false,

            // View List Command States
            remove_view_active: false,
        }
    }

    // Dashboard Command Setters
    pub fn set_refresh_panel_active(&mut self, state: bool) {
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

    pub fn set_expand_issue_active(&mut self, state: bool) {
        match self.command_bar_type {
            CommandBarType::Dashboard => {
                self.expand_issue_active = state;
            },
            _ => {
                error!("'set_expand_issue_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
                panic!("'set_expand_issue_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
            },
        }
    }

    pub fn set_modify_title_active(&mut self, state: bool) {
        match self.command_bar_type {
            CommandBarType::Dashboard => {
                self.modify_title_active = state;
            },
            _ => {
                error!("'set_modify_title_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
                panic!("'set_modify_title_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
            },
        }
    }

    pub fn set_modify_workflow_state_active(&mut self, state: bool) {
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

    pub fn set_modify_assignee_active(&mut self, state: bool) {
        match self.command_bar_type {
            CommandBarType::Dashboard => {
                self.modify_assignee_active = state;
            },
            _ => {
                error!("'set_modify_assignee_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
                panic!("'set_modify_assignee_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
            },
        }
    }

    pub fn set_modify_project_active(&mut self, state: bool) {
        match self.command_bar_type {
            CommandBarType::Dashboard => {
                self.modify_project_active = state;
            },
            _ => {
                error!("'set_modify_project_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
                panic!("'set_modify_project_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
            },
        }
    }

    pub fn set_modify_cycle_active(&mut self, state: bool) {
        match self.command_bar_type {
            CommandBarType::Dashboard => {
                self.modify_cycle_active = state;
            },
            _ => {
                error!("'set_modify_cycle_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
                panic!("'set_modify_cycle_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
            },
        }
    }


    // View List Command Setters
    pub fn set_remove_view_active(&mut self, state: bool) {
        match self.command_bar_type {
            CommandBarType::ViewList => {
                self.remove_view_active = state;
            },
            _ => {
                error!("'set_remove_view_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
                panic!("'set_remove_view_active' called on CommandBar with invalid CommandBarType: {:?}", self.command_bar_type);
            },
        };
    }

    // Determine if a Command should be styled as active or not
    pub fn get_command_style(&self, cmd: &Command) -> Style {
        match self.command_bar_type {
            CommandBarType::Dashboard => {
                match cmd {
                    Command::Dashboard(cmd) => {
                        match cmd {
                            DashboardCommand::RefreshPanel => {
                                if self.refresh_panel_active {
                                    Style::default().add_modifier(Modifier::BOLD).fg(colors::REFRESH_PANEL_CMD_ACTIVE)
                                } else {
                                    Style::default().add_modifier(Modifier::DIM).fg(colors::REFRESH_PANEL_CMD_INACTIVE)
                                }
                            },
                            DashboardCommand::ExpandIssue => {
                                if self.expand_issue_active {
                                    Style::default().add_modifier(Modifier::BOLD).fg(colors::REFRESH_PANEL_CMD_ACTIVE)
                                } else {
                                    Style::default().add_modifier(Modifier::DIM).fg(colors::REFRESH_PANEL_CMD_INACTIVE)
                                }
                            },
                            DashboardCommand::ModifyTitle => {
                                if self.expand_issue_active {
                                    Style::default().add_modifier(Modifier::BOLD).fg(colors::MODIFY_TITLE_CMD_ACTIVE)
                                } else {
                                    Style::default().add_modifier(Modifier::DIM).fg(colors::MODIFY_TITLE_CMD_INACTIVE)
                                }
                            },
                            DashboardCommand::ModifyWorkflowState => {
                                if self.modify_workflow_state_active {
                                    Style::default().add_modifier(Modifier::BOLD).fg(colors::MODIFY_WORKFLOW_STATE_CMD_ACTIVE)
                                } else {
                                    Style::default().add_modifier(Modifier::DIM).fg(colors::MODIFY_WORKFLOW_STATE_CMD_INACTIVE)
                                }
                            },
                            DashboardCommand::ModifyAssignee => {
                                if self.modify_workflow_state_active {
                                    Style::default().add_modifier(Modifier::BOLD).fg(colors::MODIFY_ASSIGNEE_CMD_ACTIVE)
                                } else {
                                    Style::default().add_modifier(Modifier::DIM).fg(colors::MODIFY_ASSIGNEE_CMD_INACTIVE)
                                }
                            },
                            DashboardCommand::ModifyProject => {
                                if self.modify_workflow_state_active {
                                    Style::default().add_modifier(Modifier::BOLD).fg(colors::MODIFY_PROJECT_CMD_ACTIVE)
                                } else {
                                    Style::default().add_modifier(Modifier::DIM).fg(colors::MODIFY_PROJECT_CMD_INACTIVE)
                                }
                            },
                            DashboardCommand::ModifyCycle => {
                                if self.modify_workflow_state_active {
                                    Style::default().add_modifier(Modifier::BOLD).fg(colors::MODIFY_CYCLE_CMD_ACTIVE)
                                } else {
                                    Style::default().add_modifier(Modifier::DIM).fg(colors::MODIFY_CYCLE_CMD_INACTIVE)
                                }
                            }
                        }
                    },
                    _ => {
                        error!("get_command_style - CommandBarType::Dashboard requires Command::Dashboard(), received: {:?}", cmd);
                        panic!("get_command_style - CommandBarType::Dashboard requires Command::Dashboard(), received: {:?}", cmd);
                    }
                }
            },
            CommandBarType::ViewList => {
                match cmd {
                    Command::ViewList(cmd) => {
                        match cmd {
                            ViewListCommand::RemoveView => {
                                if self.remove_view_active {
                                    Style::default().add_modifier(Modifier::BOLD).fg(colors::DELETE_VIEW_CMD_ACTIVE)
                                } else {
                                    Style::default().add_modifier(Modifier::DIM).fg(colors::DELETE_VIEW_CMD_INACTIVE)
                                }
                            }
                        }
                    },
                    // Error
                    _ => {
                        error!("get_command_style - CommandBarType::ViewList requires Command::ViewList(), received: {:?}", cmd);
                        panic!("get_command_style - CommandBarType::ViewList requires Command::ViewList(), received: {:?}", cmd);
                    }
                }
            }
        }
    }


    pub fn render(&self) -> Result<List, &'static str> {

        // Determine which selection of commands this Command Bar is responsible for
        let list_items: Vec<ListItem> = match self.command_bar_type {
            CommandBarType::Dashboard => {
                self.command_list.dashboard.iter()
                    .map(|e| {
                        let lines = vec![Spans::from(Span::styled(
                            e.gen_label(),
                            self.get_command_style(&e.cmd_type)
                        ))];
                        ListItem::new(lines).style(Style::default())
                    })
                    .collect()
            },
            CommandBarType::ViewList => {
                self.command_list.view_list.iter()
                    .map(|e| {
                        let lines = vec![Spans::from(Span::styled(
                            e.gen_label(),
                            self.get_command_style(&e.cmd_type)
                        ))];
                        ListItem::new(lines).style(Style::default())
                    })
                    .collect()
            }
        };

        // Create a List from all list items and highlight the currently selected one
        let items = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title("Commands"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            );
        
        Ok(items)
    }
}