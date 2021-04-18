
use serde_json::Value;
use std::sync::{Arc, Mutex};


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
}





impl Default for DashboardViewPanel {

    fn default() -> DashboardViewPanel {
        DashboardViewPanel {
            filter: Value::Null,
            issue_table_data: Arc::new(Mutex::new(None)),
        }
    }
}
