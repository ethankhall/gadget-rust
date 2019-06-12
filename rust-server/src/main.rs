extern crate hyper;
#[macro_use] extern crate clap;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate kopy_common_lib;
#[macro_use] extern crate log;
extern crate serde;
extern crate serde_yaml;
extern crate hotwatch;
extern crate chrono;

#[cfg(test)]
extern crate simple_logger;

mod config;
mod webserver;

use std::path::PathBuf;

use kopy_common_lib::configure_logging;
use clap::{App, ArgMatches};

fn run(matches: &ArgMatches) {
    let path = PathBuf::from(matches.value_of("CONFIG").unwrap());
    let bind_address = matches.value_of("bind").unwrap();
    debug!("Bind Address: {}", bind_address);
    let bind_address = bind_address.parse();
    debug!("Bind Address: {:?}", bind_address);
    webserver::run_webserver(bind_address.expect("Bind address invalid"), path);
}

fn main() {
    dotenv::dotenv().ok();

    let yml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yml)
        .version(&*format!("v{}", crate_version!()))
        .get_matches();

    configure_logging(
        matches.occurrences_of("debug") as i32,
        matches.is_present("warn"),
        matches.is_present("quite"),
    );

    match matches.subcommand() {
        ("run", Some(command_matches)) => run(command_matches),
        _ => unimplemented!()
    };
}