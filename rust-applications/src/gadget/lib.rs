extern crate hyper;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate kopy_common_lib;
#[macro_use] extern crate log;
extern crate serde;
extern crate serde_yaml;
extern crate hotwatch;
extern crate chrono;

pub mod config;
pub mod webserver;
pub mod fetcher;