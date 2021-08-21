// Represents both Column Headers and Cells
#[derive(Debug, Clone, Copy)]
pub struct TableColumn<'a> {
    pub label: &'a str,
    pub min_width: u16,
    // max line height for cells
    pub max_height: u16,

    // lower -> higher priority, after min_widths are satisfied,
    //     this specifies the order in which components will be sized according to content

    // units of priority, after min_widths are satisfied,
    //     this specifies the relative width this column should attempt to take up
    pub priority: u16,
}
/*
// "Dashboard View Configuration"
pub const DASHBOARD_VIEW_CONFIG_COLUMNS: Vec<TableColumn> = vec![
    TableColumn { label: "Name", min_width: 4, max_height: 2, priority: 2 },
    TableColumn { label: "Description", min_width: 11, max_height: 3, priority: 3 },
    TableColumn { label: "Organization", min_width: 12, max_height: 2, priority: 1 },
    TableColumn { label: "Team", min_width: 4, max_height: 2, priority: 1 }
];
*/

// "Dashboard View Configuration"
lazy_static! {
    pub static ref DASHBOARD_VIEW_CONFIG_COLUMNS: Vec<TableColumn<'static>> = {
        /*
        let mut m = Vec::new();
        m.push(TableColumn { label: "Name", min_width: 4, max_height: 2, priority: 2 });
        m.push(TableColumn { label: "Description", min_width: 11, max_height: 3, priority: 3 });
        m.push(TableColumn { label: "Organization", min_width: 12, max_height: 2, priority: 1 });
        m.push(TableColumn { label: "Team", min_width: 4, max_height: 2, priority: 1 });
        m
        */
        vec![
            TableColumn { label: "Name", min_width: 4, max_height: 2, priority: 2 },
            TableColumn { label: "Desc", min_width: 4, max_height: 3, priority: 3 },
            TableColumn { label: "Org", min_width: 3, max_height: 2, priority: 1 },
            TableColumn { label: "Team", min_width: 4, max_height: 2, priority: 1 }
        ]
    };
}
