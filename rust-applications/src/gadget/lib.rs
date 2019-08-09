extern crate hyper;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate kopy_common_lib;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate azure_sdk_core;
extern crate azure_sdk_storage_blob;
extern crate azure_sdk_storage_core;
extern crate chrono;
extern crate hotwatch;
extern crate md5;
extern crate serde;
extern crate serde_yaml;
extern crate tokio_core;

pub(crate) mod config;
pub(crate) mod manager;
pub mod prelude;
pub(crate) mod webserver;

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Should be able to make client");
}
