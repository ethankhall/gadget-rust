#![feature(extern_prelude)]
extern crate ansi_term;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate fern;
extern crate futures;
extern crate iron;
#[macro_use]
extern crate log;
extern crate router;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate tokio;
extern crate yaml_rust;

use clap::{App, AppSettings, Arg, ArgGroup, SubCommand};
use datasource::{DataSource, DataSourceContainer};
use datasource::yaml::YamlDataSource;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::timer::*;

mod datasource;
mod webserver;
mod logging;

fn main() {
    let matches = App::new("gogo-gadget")
        .about("Redirect Application for Domain Redirects")
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(Arg::with_name("debug")
            .long("debug")
            .short("d")
            .multiple(true)
            .help("Turn debugging information on.")
            .global(true)
            .conflicts_with("quite"))
        .arg(Arg::with_name("quite")
            .long("quite")
            .short("q")
            .help("Only error output will be displayed.")
            .global(true)
            .conflicts_with("debug"))
        .arg(Arg::with_name("listen")
            .long("listen")
            .help("The address to accept requests on.")
            .default_value("localhost"))
        .arg(Arg::with_name("port")
            .long("port")
            .help("The port to accept requests on.")
            .default_value("3000"))
        .group(ArgGroup::with_name("logging")
            .args(&["debug", "quite"]))
        .group(ArgGroup::with_name("webserver")
            .args(&["listen", "port"]))
        .subcommand(SubCommand::with_name("yaml")
            .setting(AppSettings::ArgRequiredElseHelp)
            .about("Uses YAML as the data source")
            .arg(Arg::with_name("path")
                .takes_value(true)
                .help("Path to the Yaml File")))
        .get_matches();

    logging::configure_logging(
        matches.occurrences_of("debug") as i32,
        matches.is_present("quite"),
    );

    let mut runtime = match Runtime::new() {
        Ok(x) => x,
        Err(err) => {
            error!("Unable to start background thread because {:?}. Dieing. Goodbye!", err);
            panic!("Unable to start background thread.")
        }
    };

    let datasource = match matches.subcommand() {
        ("yaml", Some(yaml_matches)) => {
            let path = PathBuf::from(yaml_matches.value_of("path").unwrap());
            info!("Loading YAML definition from {:#?}", path);
            match YamlDataSource::new(path) {
                Ok(source) => {
                    Ok(DataSourceContainer { data_source: Box::new(source) })
                }
                Err(v) => Err(v)
            }
        }
        _ => panic!(), // Assuming you've listed all direct children above, this is unreachable
    };

    match datasource {
        Err(e) => panic!("Unable to get datasource! `{:?}`", e),
        Ok(source) => run_web_server(matches, &mut runtime, Arc::new(source))
    };
}

fn run_web_server(matches: clap::ArgMatches, runtime: &mut Runtime, container: Arc<DataSourceContainer>) {
    let when = Instant::now() + Duration::from_millis(300);
    let interval = Interval::new(when, Duration::from_secs(60));
    let task = RefreshFuture { datasource: container.clone(), interval }
        .for_each(move |source| {
            match source.reload() {
                Ok(()) => info!("Reloaded yaml"),
                Err(err) => warn!("Unable to reload YAML because {:?}", err)
            };
            Ok(())
        })
        .map_err(|_| error!("Error updating"));

    runtime.spawn(task);

    let listen = matches.value_of("listen")
        .expect("The value will always be set (because of default)");

    let port = value_t!(matches, "port", u32).unwrap();

    webserver::exec_webserver(listen, port, container.clone())
}

struct RefreshFuture {
    datasource: Arc<DataSourceContainer>,
    interval: Interval,
}

impl Stream for RefreshFuture {
    type Item = Arc<DataSourceContainer>;
    type Error = tokio::timer::Error;

    fn poll(&mut self) -> Result<Async<Option<<Self as Stream>::Item>>, <Self as Stream>::Error> {
        return match self.interval.poll() {
            Ok(Async::Ready(_)) => Ok(Async::Ready(Some(self.datasource.clone()))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(x) => Err(x)
        };
    }
}