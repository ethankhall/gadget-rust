extern crate hyper;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate kopy_common_lib;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
extern crate serde;
extern crate serde_yaml;
extern crate hotwatch;
extern crate chrono;

pub mod config;
pub mod webserver;
pub mod manager;

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Should be able to make client");
}