
use serde_json::Value;
use std::sync::{Arc, Mutex};
use super::linear_issue_display::LinearIssueDisplay;
use crate::util::ui::{ TableStyle };

use tui::{
    widgets::{ Table, TableState},
};


#[derive(Debug, Clone)]
pub struct DashboardViewPanel {
    pub filter: Value,
    pub issue_table_data: Arc<Mutex<Option<Value>>>,
}

impl DashboardViewPanel {
    pub fn with_filter(f: Value) -> DashboardViewPanel {
        DashboardViewPanel {
            filter: f,
            issue_table_data: Arc::new(Mutex::new(None)),
        }
    }

    pub fn render<'a>(data: &'a Option<Value>, filter: &Value, view_idx: u16) -> Result<Table<'a>, &'static str> {
        // Create TableStyle from filter
        let table_style = TableStyle { title_style: Some(( filter["name"].clone(), filter["color"].clone() )),
                                        row_bottom_margin: Some(0),
                                        view_idx: Some(view_idx),
                                    };

        LinearIssueDisplay::get_rendered_issue_data(&data, table_style)
    }
}





impl Default for DashboardViewPanel {

    fn default() -> DashboardViewPanel {
        DashboardViewPanel {
            filter: Value::Null,
            issue_table_data: Arc::new(Mutex::new(None)),
        }
    }
}
