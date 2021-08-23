use tui::widgets::ListState;

pub mod event;
pub mod state_list;
pub mod state_table;

pub mod ui;

pub mod colors;

mod cursor;
pub use cursor::GraphQLCursor;

mod query;
pub use query::set_linear_after_cursor_from_opt;
pub use query::verify_linear_api_key;

mod dashboard;
pub use dashboard::fetch_selected_view_panel_issue;
pub use dashboard::fetch_selected_workflow_state;

pub mod command_list;
pub mod layout;
pub mod str;
pub mod loader;



// StatefulList with non-instance methods
#[derive(Debug)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn selected(mut self) -> StatefulList<T> {
        self.next();
        self
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }


    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

}