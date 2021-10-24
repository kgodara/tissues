pub mod table_columns;
pub mod command_list;
pub mod colors;

pub const LINEAR_TOKEN_LEN: u16=48;
pub const SCROLL_TICK_MAX: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IssueModificationOp {
    // implemented
    Title,
    WorkflowState,
    Assignee,
    Project,
    Cycle,
    Team,

    // unimplemented
    Labels,
}