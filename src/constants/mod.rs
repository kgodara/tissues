pub mod table_columns;
pub mod command_list;
pub mod colors;

#[derive(Debug, Clone, Copy)]
pub enum IssueModificationOp {
    ModifyWorkflowState,
    ModifyAssignee,
    ModifyLabels,
}