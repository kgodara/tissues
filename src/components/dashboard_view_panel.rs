
use serde_json::Value;
use std::sync::{Arc, Mutex};
use super::linear_issue_display::LinearIssueDisplay;
use crate::util::ui::{ TableStyle };
use crate::linear::view_resolver::ViewLoader;

use tui::{
    layout::{ Constraint },
    widgets::{ Table },
};


#[derive(Debug, Clone)]
pub struct DashboardViewPanel {
    pub filter: Value,
    pub issue_table_data: Arc<Mutex<Vec<Value>>>,
    pub view_loader: Arc<Mutex<Option<ViewLoader>>>,
    pub request_num: Arc<Mutex<u32>>,
    pub loading: Arc<Mutex<bool>>,
}

impl DashboardViewPanel {
    pub fn with_filter(f: Value) -> DashboardViewPanel {
        DashboardViewPanel {
            filter: f,
            issue_table_data: Arc::new(Mutex::new(Vec::new())),
            view_loader: Arc::new(Mutex::new(None)),
            request_num: Arc::new(Mutex::new(0)),
            loading: Arc::new(Mutex::new(false)),
        }
    }

    pub fn render<'a>(data: &'a [Value],
        filter: &Value,
        widths: &Vec<Constraint>,
        table_style: TableStyle
        ) -> Result<Table<'a>, &'static str> {

        LinearIssueDisplay::get_rendered_issue_data(data, widths, table_style)
    }
}





impl Default for DashboardViewPanel {

    fn default() -> DashboardViewPanel {
        DashboardViewPanel {
            filter: Value::Null,
            issue_table_data: Arc::new(Mutex::new(Vec::new())),
            view_loader: Arc::new(Mutex::new(None)),
            request_num: Arc::new(Mutex::new(0)),
            loading: Arc::new(Mutex::new(false)),
        }
    }
}
