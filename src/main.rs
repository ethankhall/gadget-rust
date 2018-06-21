#![feature(extern_prelude)]

#[macro_use]
extern crate clap;

extern crate yaml_rust;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate fern;
#[macro_use]
extern crate log;
extern crate iron;
extern crate router;
extern crate chrono;
extern crate ansi_term;

mod datasource;
mod webserver;
mod logging;

use std::path::PathBuf;
use datasource::DataSourceContainer;

fn main() {
    let matches = clap_app!(myapp =>
        (@setting SubcommandRequiredElseHelp)
        (about: "Redirect Application for Domain Redirects")
        (@group logging =>
            (@arg debug: -d ... +global "Turn debugging information on")
            (@arg quite: -q --quite +global "Only error output will be displayed")
        )
        (@subcommand yaml =>
            (@setting ArgRequiredElseHelp)
            (about: "Loads data from Yaml")
            (@arg path: +takes_value "Path to the Yaml File")
        )
    ).get_matches();

    logging::configure_logging(
        matches.occurrences_of("debug") as i32,
        matches.is_present("quite"),
    );

    let datasource = match matches.subcommand() {
        ("yaml", Some(yaml_matches)) => {
            let path = PathBuf::from(yaml_matches.value_of("path").unwrap());
            info!("Loading YAML definition from {:#?}", path);
            match datasource::yaml::YamlDataSource::new(path) {
                Ok(source) => Ok(DataSourceContainer { data_source: Box::new(source) }),
                Err(v) => Err(v)
            }
        }
        _ => panic!(), // Assuming you've listed all direct children above, this is unreachable
    };

    match datasource {
        Err(e) => panic!("Unable to get datasource! `{:?}`", e),
        Ok(source) => webserver::exec_webserver(source)
    };
}
