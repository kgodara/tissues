use tui::widgets::TableState;
// A set of utilities for working with a ListState and an Iterator

pub fn next<T>(state: &mut TableState, items: &Vec<T>) {
    let i = match state.selected() {
        Some(i) => {
            if i >= items.len() - 1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}

pub fn with_next<T>(state: &TableState, items: &Vec<T>) -> TableState {
    let i = match state.selected() {
        Some(i) => {
            if i >= items.len() - 1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    };
    let mut state = TableState::default();
    state.select(Some(i));
    state 
}

pub fn with_previous<T>(state: &TableState, items: &Vec<T>) -> TableState {
    let i = match state.selected() {
        Some(i) => {
            if i == 0 {
                items.len() - 1
            } else {
                i - 1
            }
        }
        None => 0,
    };
    let mut state = TableState::default();
    state.select(Some(i));
    state
}

pub fn is_last_element<T>(state: &TableState, items: &Vec<T>) -> bool {
    let i = match state.selected() {
        Some(x) => {
            if x >= items.len() - 1 {
                0 as i32
            } else {
                (x + 1) as i32
            }
        }
        None => -1 as i32,
    };
    if i == 0 {
        return true;
    }
    return false;
}

pub fn previous<T>(state: &mut TableState, items: &Vec<T>) {

    let i = match state.selected() {
        Some(i) => {
            if i == 0 {
                items.len() - 1
            } else {
                i - 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}

pub fn unselect(state: &mut TableState) {
    state.select(None);
}