// pub mod event;
pub mod event_crossterm;

pub mod stateful_list;

pub mod list_state;
pub mod table_state;

pub mod ui;

mod cursor;
pub use cursor::GraphQLCursor;

pub mod dashboard;

pub mod layout;
pub mod str;
pub mod loader;

pub mod table;

#[macro_export]
macro_rules! error_panic {
    ($($arg:tt)*) => {
        error!($($arg)*);
        panic!($($arg)*);
    };
}
pub use error_panic;