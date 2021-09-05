use std::cmp::max;
use std::sync::{
    Arc,
    Mutex,
    atomic::AtomicBool,
};

use tui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use serde_json::{ Value, json, Map };


use crate::linear::{
    client::LinearClient,
    LinearConfig
};

use crate::util::{
    table::{ values_to_str, format_cell_fields,
        get_row_height, colored_cell,
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

pub struct LinearIssueOpInterface {

    pub current_op: IssueModificationOp,
    pub selected_idx: Option<usize>,
    pub data_state: TableState,
    pub loading: Arc<AtomicBool>,
    pub cursor: Arc<Mutex<GraphQLCursor>>,

    pub workflow_states_data: Arc<Mutex<Vec<Value>>>,
    pub users_data: Arc<Mutex<Vec<Value>>>,
    pub projects_data: Arc<Mutex<Vec<Value>>>,
    pub cycles_data: Arc<Mutex<Vec<Value>>>,
}


impl LinearIssueOpInterface {

    // loading functions section start

    pub async fn load_op_data(op: &IssueModificationOp,
        linear_config: LinearConfig,
        linear_cursor: Option<GraphQLCursor>,
        team: &Value)-> Option<Value> {

        let team_id;
        if let Some(x) = team.as_str() {
            team_id = x;
        }
        else {
            return None;
        }

        let mut variables: Map<String, Value> = Map::new();
        variables.insert(String::from("ref"), Value::String(String::from(team_id)));

        debug!("load_op_data about to dispatch query");

        let data_result = match op {
            IssueModificationOp::ModifyWorkflowState => {
                LinearClient::get_workflow_states_by_team(linear_config, linear_cursor, variables).await
            }
            IssueModificationOp::ModifyAssignee => {
                LinearClient::get_users_by_team(linear_config, linear_cursor, variables).await
            },
            IssueModificationOp::ModifyProject => {
                LinearClient::get_projects_by_team(linear_config, linear_cursor, variables).await
            },
            IssueModificationOp::ModifyCycle => {
                LinearClient::get_cycles_by_team(linear_config, linear_cursor, variables).await
            },
            _ => {
                error!("LinearIssueOpInterface::load_op_data, invalid IssueModificationOp: {:?}", op);
                panic!("LinearIssueOpInterface::load_op_data, invalid IssueModificationOp: {:?}", op);
            }
        };

        let data: Value;
        let cursor_info: Value;

        match data_result {
            Ok(x) => {
                data = x["data_nodes"].clone();
                cursor_info = x["cursor_info"].clone();
            },
            Err(y) => {
                error!("Get data for {:?} failed: {:?}", op, y);
                return None;
            },
        }

        debug!("load_op_data - op, data: {:?}, {:?}", op, data);

        if data == Value::Null {
            return Some(Value::Array(vec![]));
        }

        match data {
            Value::Array(_) => {
                return Some(json!( { "data": data, "cursor_info": cursor_info } ));
            },
            _ => {return None;},
        }
    }

    // loading functions section end

    pub fn table_data_from_op(&self) -> Arc<Mutex<Vec<Value>>> {
        match self.current_op {
            IssueModificationOp::ModifyWorkflowState => {
                self.workflow_states_data.clone()
            },
            IssueModificationOp::ModifyAssignee => {
                self.users_data.clone()
            },
            IssueModificationOp::ModifyProject => {
                self.projects_data.clone()
            },
            IssueModificationOp::ModifyCycle => {
                self.cycles_data.clone()
            }
            _ => {panic!("Not ready")}
        }
    }

    pub fn is_valid_selection_for_update(&self) -> bool {
        match self.current_op {
            IssueModificationOp::ModifyWorkflowState => {
                self.selected_idx.is_some()
            },
            IssueModificationOp::ModifyAssignee => {
                self.selected_idx.is_some()
            },
            IssueModificationOp::ModifyProject => {
                self.selected_idx.is_some()
            },
            IssueModificationOp::ModifyCycle => {
                self.selected_idx.is_some()
            },
            _ => {panic!("not ready")}
        }
    }

    pub fn reset_op(&mut self) {
        match self.current_op {
            IssueModificationOp::ModifyWorkflowState => {
                self.workflow_states_data = Arc::new(Mutex::new(vec![]));
                self.selected_idx = None;
            },
            IssueModificationOp::ModifyAssignee => {
                self.users_data = Arc::new(Mutex::new(vec![]));
                self.selected_idx = None;
            },
            IssueModificationOp::ModifyProject => {
                self.projects_data = Arc::new(Mutex::new(vec![]));
                self.selected_idx = None;
            },
            IssueModificationOp::ModifyCycle => {
                self.cycles_data = Arc::new(Mutex::new(vec![]));
                self.selected_idx = None;
            },
            _ => {panic!("Not ready")}
        };
        self.data_state = TableState::default();
        self.cursor = Arc::new(Mutex::new(GraphQLCursor::default()));
    }

    // render helper functions
    fn cell_fields_from_row(op: IssueModificationOp, widths: &[Constraint], row: &Value) -> Vec<String> {
        let cell_fields: Vec<String>;
        match op {
            IssueModificationOp::ModifyWorkflowState => {
                cell_fields = values_to_str(
                    &[
                        row["name"].clone(),
                        row["type"].clone(),
                        row["description"].clone()
                    ],
                    &WORKFLOW_STATE_SELECT_COLUMNS
                );

                // Get the formatted Strings for each cell field
                format_cell_fields(&cell_fields, widths, &WORKFLOW_STATE_SELECT_COLUMNS)
            },
            IssueModificationOp::ModifyAssignee => {
                cell_fields = values_to_str(
                    &[
                        row["name"].clone(),
                        row["displayName"].clone(),
                    ],
                    &ASSIGNEE_SELECT_COLUMNS
                );

                format_cell_fields(&cell_fields, widths, &ASSIGNEE_SELECT_COLUMNS)
            },
            IssueModificationOp::ModifyProject => {
                cell_fields = values_to_str(
                    &[
                        row["name"].clone(),
                        row["state"].clone(),
                    ],
                    &PROJECT_SELECT_COLUMNS
                );

                format_cell_fields(&cell_fields, widths, &PROJECT_SELECT_COLUMNS)
            },
            IssueModificationOp::ModifyCycle => {
                cell_fields = values_to_str(
                    &[
                        row["name"].clone(),
                        row["number"].clone(),
                        row["startsAt"].clone(),
                        row["endsAt"].clone(),
                    ],
                    &CYCLE_SELECT_COLUMNS
                );

                format_cell_fields(&cell_fields, widths, &CYCLE_SELECT_COLUMNS)
            },
            _ => {
                panic!("Not ready yet");
            }
        }
    }

    pub fn widths_from_rect_op(bbox: &Rect, op: &IssueModificationOp) -> Vec<Constraint> {
        match op {
            IssueModificationOp::ModifyWorkflowState => {
                widths_from_rect(bbox, &WORKFLOW_STATE_SELECT_COLUMNS)
            },
            IssueModificationOp::ModifyAssignee => {
                widths_from_rect(bbox, &ASSIGNEE_SELECT_COLUMNS)
            },
            IssueModificationOp::ModifyProject => {
                widths_from_rect(bbox, &PROJECT_SELECT_COLUMNS)
            },
            IssueModificationOp::ModifyCycle => {
                widths_from_rect(bbox, &CYCLE_SELECT_COLUMNS)
            },
            _ => {panic!("Not ready")}
        }
    }

    pub fn title_from_op(op: &IssueModificationOp) -> String {
        match op {
            IssueModificationOp::ModifyWorkflowState => {
                "Select New Workflow State".to_string()
            },
            IssueModificationOp::ModifyAssignee => {
                "Select New Assignee".to_string()
            },
            IssueModificationOp::ModifyProject => {
                "Select New Project".to_string()
            },
            IssueModificationOp::ModifyCycle => {
                "Select New Cycle".to_string()
            },
            _ => {
                panic!("Not ready");
            }
        }
    }

    pub fn render<'a>(
        op: IssueModificationOp,
        table_data: &[Value],
        widths: &[Constraint],
        table_style: TableStyle) -> Result<Table<'a>, &'static str> {

        let bottom_margin = table_style.row_bottom_margin.unwrap_or(0);

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::DarkGray);


        let header_cells: Vec<Cell> = match op {
                IssueModificationOp::ModifyWorkflowState => { &*WORKFLOW_STATE_SELECT_COLUMNS },
                IssueModificationOp::ModifyAssignee => { &*ASSIGNEE_SELECT_COLUMNS },
                IssueModificationOp::ModifyProject => { &*PROJECT_SELECT_COLUMNS },
                IssueModificationOp::ModifyCycle => { &*CYCLE_SELECT_COLUMNS },
                _ => { 
                    error!("LinearIssueOpInterface::render - header_cells invalid IssueModificationOp: {:?}", op);
                    panic!("LinearIssueOpInterface::render - header_cells invalid IssueModificationOp: {:?}", op);
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

        let mut rows: Vec<Row> = table_data.iter()
            .map(|row| {

                let cell_fields_formatted = LinearIssueOpInterface::cell_fields_from_row(op, widths, row);

                max_seen_row_size = max(get_row_height(&cell_fields_formatted), max_seen_row_size);

                let mut cells: Vec<Cell> = cell_fields_formatted
                    .iter()
                    .map(|c| Cell::from(c.clone()))
                    .collect();

                // gen relevant cell colored & replace uncolored edition with colored
                match op {
                    IssueModificationOp::ModifyWorkflowState => {
        
                        let name: String = cell_fields_formatted[0].clone();
                        let color = row["color"].clone();
        
                        // Insert new "name" cell, and remove unformatted version
                        cells.insert(0, colored_cell(name, color));
                        cells.remove(1);
                    },
                    IssueModificationOp::ModifyAssignee => {
                        // No colored cell for users
                    },
                    IssueModificationOp::ModifyProject => {
                        let name: String = cell_fields_formatted[0].clone();
                        let color = row["color"].clone();
        
                        // Insert new "name" cell, and remove unformatted version
                        cells.insert(0, colored_cell(name, color));
                        cells.remove(1);
                    },
                    IssueModificationOp::ModifyCycle => {
                        // No colored cell for cycles
                    },
                    _ => {
                        panic!("Not ready yet");
                    }
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
            current_op: IssueModificationOp::ModifyWorkflowState,
            selected_idx: None,
            data_state: TableState::default(),
            loading: Arc::new(AtomicBool::new(false)),
            cursor: Arc::new(Mutex::new(GraphQLCursor::default())),

            workflow_states_data: Arc::new(Mutex::new(vec![])),
            users_data: Arc::new(Mutex::new(vec![])),
            projects_data: Arc::new(Mutex::new(vec![])),
            cycles_data: Arc::new(Mutex::new(vec![])),
        }
    }
}