// TODO: Remove these
#![allow(unused_variables)]
#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::error::Error;

mod backend;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    // We use the fast_log crate for logging.
    //
    // TODO: Ability to log to a file.
    #[cfg(debug_assertions)]
    fast_log::init(fast_log::Config::new().console().level(log::LevelFilter::Debug)).unwrap();
    #[cfg(not(debug_assertions))]
    fast_log::init(fast_log::Config::new().console().level(log::LevelFilter::Info)).unwrap();

    ui::run_ui()
}
