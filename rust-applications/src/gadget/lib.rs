extern crate hyper;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate kopy_common_lib;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
extern crate serde;
extern crate serde_yaml;
extern crate hotwatch;
extern crate chrono;
extern crate azure_sdk_storage_blob;
extern crate azure_sdk_core;
extern crate azure_sdk_storage_core;
extern crate tokio_core;
extern crate md5;

pub mod prelude;
pub(crate) mod config;
pub(crate) mod webserver;
pub(crate) mod manager;

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Should be able to make client");
}