use std::cmp::max;
use std::sync::{
    Arc,
    Mutex,
    atomic::AtomicBool,
};

use unicode_segmentation::UnicodeSegmentation;

use tui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::app::Platform;


use crate::linear::{
    client::{ IssueFieldObject },
    schema::{Cycle, Project, TeamMember, State},
};

use crate::util::{
    error_panic,
    table::{ empty_str_to_fallback, format_cell_fields,
        get_row_height, row_min_render_height, colored_cell,
        TableStyle, gen_table_title_spans,
    },
    layout::{
        widths_from_rect
    },
    GraphQLCursor
};

use crate::constants::{
    IssueModificationOp,
    colors,
    table_columns::{
        WORKFLOW_STATE_SELECT_COLUMNS, ASSIGNEE_SELECT_COLUMNS,
        PROJECT_SELECT_COLUMNS, CYCLE_SELECT_COLUMNS
    }
};

#[derive(Debug, Default, Clone)]
pub struct ModificationOpData {
    pub cycles: Vec<Cycle>,
    pub projects: Vec<Project>,
    pub users: Vec<TeamMember>,
    pub workflow_states: Vec<State>,
}

pub struct LinearIssueOpInterface {

    pub current_op: Option<IssueModificationOp>,
    pub selected_idx: Option<usize>,
    pub data_state: TableState,
    pub loading: Arc<AtomicBool>,
    pub cursor: Arc<Mutex<GraphQLCursor>>,

    pub obj_data: Arc<Mutex<ModificationOpData>>,
}


impl LinearIssueOpInterface {

    pub fn table_data_from_op(&self) -> Option<Vec<IssueFieldObject>> {
        let obj_data_lock = self.obj_data.lock().unwrap();
        match self.current_op {
            Some(IssueModificationOp::WorkflowState) => {
                Some(obj_data_lock.workflow_states
                    .iter()
                    .map(|state| { IssueFieldObject::State(state.clone()) })
                    .collect())
            },
            Some(IssueModificationOp::Assignee) => {
                Some(obj_data_lock.users
                    .iter()
                    .map(|member| { IssueFieldObject::TeamMember(member.clone()) })
                    .collect())
            },
            Some(IssueModificationOp::Project) => {
                Some(obj_data_lock.projects
                    .iter()
                    .map(|project| { IssueFieldObject::Project(project.clone()) })
                    .collect())
            },
            Some(IssueModificationOp::Cycle) => {
                Some(obj_data_lock.cycles
                    .iter()
                    .map(|cycle| { IssueFieldObject::Cycle(cycle.clone()) })
                    .collect())
            },
            _ => { None }
        }
    }


    pub fn is_valid_selection_for_update(&self, title_input: &str) -> bool {
        match self.current_op {
            Some(IssueModificationOp::Title) => {
                title_input.graphemes(true).count() > 0
            },
            Some(IssueModificationOp::WorkflowState) => {
                self.selected_idx.is_some()
            },
            Some(IssueModificationOp::Assignee) => {
                self.selected_idx.is_some()
            },
            Some(IssueModificationOp::Project) => {
                self.selected_idx.is_some()
            },
            Some(IssueModificationOp::Cycle) => {
                self.selected_idx.is_some()
            },
            _ => {
                false
            }
        }
    }

    pub fn reset_op(&mut self) {
        let mut obj_data_lock = self.obj_data.lock().unwrap();
        match self.current_op {
            Some(IssueModificationOp::Title) => {

            },
            Some(IssueModificationOp::WorkflowState) => {
                obj_data_lock.workflow_states = Vec::default();
            },
            Some(IssueModificationOp::Assignee) => {
                obj_data_lock.users = Vec::default();
            },
            Some(IssueModificationOp::Project) => {
                obj_data_lock.projects = Vec::default();
            },
            Some(IssueModificationOp::Cycle) => {
                obj_data_lock.cycles = Vec::default();
            },
            _ => {
                error_panic!("reset_op: invalid LinearIssueOpInterface::current_op: {:?}", self.current_op);
            }
        };

        self.selected_idx = None;
        self.current_op = None;

        self.data_state = TableState::default();
        self.cursor = Arc::new(Mutex::new(GraphQLCursor::with_platform(Platform::Linear)));
    }

    // render helper functions
    fn cell_fields_from_row(row: &IssueFieldObject, widths: &[Constraint]) -> Vec<String> {

        let columns_from_row = |row| match row {
            IssueFieldObject::Cycle(_) => &*CYCLE_SELECT_COLUMNS,
            IssueFieldObject::Project(_) => &*PROJECT_SELECT_COLUMNS,
            IssueFieldObject::TeamMember(_) => &*ASSIGNEE_SELECT_COLUMNS,
            IssueFieldObject::State(_) => &*WORKFLOW_STATE_SELECT_COLUMNS,
        };

        let values = match row {
            IssueFieldObject::State(state) => {
                vec![
                    state.name.as_str(),
                    state.type_.as_str(),
                    state.description.as_deref().unwrap_or(""),
                ]
            },
            IssueFieldObject::TeamMember(member) => {
                vec![
                    member.name.as_str(),
                    member.display_name.as_str(),
                ]
            },
            IssueFieldObject::Project(project) => {
                vec![
                    project.name.as_str(),
                    project.state.as_str(),
                ]
            },
            IssueFieldObject::Cycle(cycle) => {
                vec![
                    cycle.name.as_deref().unwrap_or(""),
                    //cycle.number.to_string().as_str(),
                    cycle.starts_at.as_str(),
                    cycle.ends_at.as_str(),
                ]
            },
        };

        let cell_fields: Vec<String> = empty_str_to_fallback(&values[..], columns_from_row(row.clone()));

        let row_height = row_min_render_height(&cell_fields, widths, columns_from_row(row.clone()));
        format_cell_fields(&cell_fields, widths, columns_from_row(row.clone()), Some(row_height))

    }

    pub fn widths_from_rect_op(bbox: &Rect, op: &IssueModificationOp) -> Vec<Constraint> {

        widths_from_rect(
            bbox,
    match op {
                IssueModificationOp::WorkflowState => &WORKFLOW_STATE_SELECT_COLUMNS,
                IssueModificationOp::Assignee => &ASSIGNEE_SELECT_COLUMNS,
                IssueModificationOp::Project => &PROJECT_SELECT_COLUMNS,
                IssueModificationOp::Cycle => &CYCLE_SELECT_COLUMNS,
                _ => panic!("Not ready")
            }
        )
    }

    pub fn title_from_op(op: &IssueModificationOp) -> String {
        match op {
            IssueModificationOp::WorkflowState => {
                "Select New Workflow State".to_string()
            },
            IssueModificationOp::Assignee => {
                "Select New Assignee".to_string()
            },
            IssueModificationOp::Project => {
                "Select New Project".to_string()
            },
            IssueModificationOp::Cycle => {
                "Select New Cycle".to_string()
            },
            _ => {
                panic!("Not ready");
            }
        }
    }

    pub fn render<'a>(
        op: IssueModificationOp,
        table_data: &ModificationOpData,
        widths: &[Constraint],
        table_style: TableStyle) -> Result<Table<'a>, &'static str> {

        let bottom_margin = table_style.row_bottom_margin.unwrap_or(0);

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::DarkGray);


        let header_cells: Vec<Cell> = match op {
                IssueModificationOp::WorkflowState => { &*WORKFLOW_STATE_SELECT_COLUMNS },
                IssueModificationOp::Assignee => { &*ASSIGNEE_SELECT_COLUMNS },
                IssueModificationOp::Project => { &*PROJECT_SELECT_COLUMNS },
                IssueModificationOp::Cycle => { &*CYCLE_SELECT_COLUMNS },
                _ => {
                    error_panic!("LinearIssueOpInterface::render - header_cells invalid IssueModificationOp: {:?}", op);
                }
            }
            .iter()
            .map(|h| Cell::from(&*h.label).style(Style::default().fg(Color::LightGreen)))
            .collect();

        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        let mut max_seen_row_size: usize = 0;

        // TODO: This approach seems goofy
        let obj_vec_to_iter: Vec<IssueFieldObject> = match op {
            IssueModificationOp::WorkflowState => {
                table_data.workflow_states
                    .iter()
                    .map(|state| { IssueFieldObject::State(state.clone()) })
                    .collect()
            },
            IssueModificationOp::Assignee => {
                table_data.users
                    .iter()
                    .map(|user| { IssueFieldObject::TeamMember(user.clone()) })
                    .collect()
            },
            IssueModificationOp::Project => {
                table_data.projects
                    .iter()
                    .map(|project| { IssueFieldObject::Project(project.clone()) })
                    .collect()
            },
            IssueModificationOp::Cycle => {
                table_data.cycles
                    .iter()
                    .map(|cycle| { IssueFieldObject::Cycle(cycle.clone()) })
                    .collect()
            },
            _ => {
                panic!("unsupported op!");
            },
        };

        let mut rows: Vec<Row> = obj_vec_to_iter.iter()
            .map(|row| {

                let cell_fields_formatted = LinearIssueOpInterface::cell_fields_from_row(row, widths);
                let name_str: String = cell_fields_formatted[0].clone();

                max_seen_row_size = max(get_row_height(&cell_fields_formatted), max_seen_row_size);

                let mut cells: Vec<Cell> = cell_fields_formatted
                    .iter()
                    .map(|c| Cell::from(c.clone()))
                    .collect();

                // gen relevant cell colored & replace uncolored edition with colored
                match row {
                    IssueFieldObject::Cycle(_cycle) => {},
                    IssueFieldObject::Project(project) => { cells[0] = colored_cell(name_str, &project.color); },
                    IssueFieldObject::TeamMember(_member) => {},
                    IssueFieldObject::State(state) => { cells[0] = colored_cell(name_str, &state.color); },
                };
    
                Row::new(cells)
                    .bottom_margin(bottom_margin)
                    .style(Style::default().fg(colors::ISSUE_MODIFICATION_TABLE_TITLE))
            })
            .collect();

        // Set all row heights to max_seen_row_size
        rows = rows.into_iter()
            .map(|row| {
                row.height(max_seen_row_size as u16)
            })
            .collect();

        let table_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if table_style.highlight_table { Color::Yellow } else { Color::White }))
            .title( gen_table_title_spans(table_style) );

        let t = Table::new(rows)
            .header(header)
            .block(table_block)
            .highlight_style(selected_style);

        Ok(t)
    }
}



impl Default for LinearIssueOpInterface {
    fn default() -> LinearIssueOpInterface {
        LinearIssueOpInterface {
            current_op: None,
            selected_idx: None,
            data_state: TableState::default(),
            loading: Arc::new(AtomicBool::new(false)),
            cursor: Arc::new(Mutex::new(GraphQLCursor::with_platform(Platform::Linear))),

            obj_data: Arc::new(Mutex::new(ModificationOpData::default())),
        }
    }
}