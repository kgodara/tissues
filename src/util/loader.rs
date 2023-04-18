use crate::util::error_panic;

pub const LOADER_STATE_MAX: u16 = 3u16;

pub fn loader_from_state(loading: bool, state: u16) -> char {

    if !loading {
        return '✓';
    }

    if !(0..LOADER_STATE_MAX).contains(&state) {
        error_panic!("loader_from_state - invalid state - LOADER_STATE_MAX, state: {:?}, {:?}", LOADER_STATE_MAX, state);
    }

    match state {
        0 => '◐',
        1 => '◓',
        2 => '◑',
        3 => '◒',
        _ => '!',
    }
}