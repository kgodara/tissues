use tui::widgets::ListState;
// A set of utilities for working with a ListState and an Iterator

pub fn next<T>(state: &mut ListState, items: &[T]) {
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

pub fn previous<T>(state: &mut ListState, items: &[T]) {

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

pub fn unselect(state: &mut ListState) {
    state.select(None);
}