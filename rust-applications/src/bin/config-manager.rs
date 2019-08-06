extern crate hyper;
#[macro_use] extern crate clap;
extern crate serde;
extern crate serde_yaml;
extern crate hotwatch;
extern crate chrono;
extern crate gadget;

#[cfg(test)]
extern crate simple_logger;

use std::path::PathBuf;

use kopy_common_lib::configure_logging;
use clap::App;

use gadget::manager::fetch::run_fetcher;
use gadget::manager::pusher::run_pusher;

fn main() {
    dotenv::dotenv().ok();

    let yml = load_yaml!("config.yaml");
    let matches = App::from_yaml(yml)
        .version(&*format!("v{}", crate_version!()))
        .get_matches();

    configure_logging(
        matches.occurrences_of("debug") as i32,
        matches.is_present("warn"),
        matches.is_present("quite"),
    );

    let fetch_config_path = PathBuf::from(matches.value_of("FETCH_CONFIG").unwrap());
    let dest = PathBuf::from(matches.value_of("DEST").unwrap());

    match matches.subcommand() {
        ("poll", Some(_)) => run_fetcher(fetch_config_path, dest),
        ("push", Some(_)) => run_pusher(fetch_config_path, dest),
        _ => unimplemented!()
    };
}