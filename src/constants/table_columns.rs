// Represents both Column Headers and Cells
#[derive(Debug, Clone, Copy)]
pub struct TableColumn<'a> {
    pub label: &'a str,
    pub min_width: u16,
    // max line height for cells
    pub max_height: u16,

    // units of priority, after min_widths are satisfied,
    //     this specifies the relative width this column should attempt to take up
    pub priority: u16,
}


lazy_static! {

    pub static ref VIEW_PANEL_COLUMNS: Vec<TableColumn<'static>> = {
        vec![
            TableColumn { label: "#", min_width: 4, max_height: 1, priority: 1 },
            TableColumn { label: "Title", min_width: 5, max_height: 2, priority: 3 },
            TableColumn { label: "State", min_width: 5, max_height: 1, priority: 2 },
            TableColumn { label: "Desc", min_width: 4, max_height: 3, priority: 3 },
            TableColumn { label: "createdAt", min_width: 9, max_height: 1, priority: 2 },
        ]
    };

    pub static ref DASHBOARD_VIEW_CONFIG_COLUMNS: Vec<TableColumn<'static>> = {
        vec![
            TableColumn { label: "Name", min_width: 4, max_height: 2, priority: 2 },
            TableColumn { label: "Desc", min_width: 4, max_height: 3, priority: 3 },
            TableColumn { label: "Org", min_width: 3, max_height: 2, priority: 1 },
            TableColumn { label: "Team", min_width: 4, max_height: 2, priority: 1 }
        ]
    };

    pub static ref CUSTOM_VIEW_SELECT_COLUMNS: Vec<TableColumn<'static>> = {
        vec![
            TableColumn { label: "Name", min_width: 4, max_height: 2, priority: 2 },
            TableColumn { label: "Desc", min_width: 4, max_height: 3, priority: 3 },
            TableColumn { label: "Org", min_width: 3, max_height: 2, priority: 1 },
            TableColumn { label: "Team", min_width: 4, max_height: 2, priority: 1 }
        ]
    };
}
