
#[macro_use]
extern crate lazy_static;


#[macro_use] extern crate log;
extern crate simplelog;

use simplelog::*;

pub mod app;
pub mod graphql;
pub mod linear;
pub mod ui;
pub mod constants;
pub mod util;
pub mod errors;
pub mod command;
pub mod network;

pub mod components;

/*
pub fn test_lib_func(a: u8, b: u8) -> Option<u8> {
    Some(a + b)
}
*/