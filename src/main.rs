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
extern crate tokio;
extern crate futures;

mod datasource;
mod webserver;
mod logging;

use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::sync::Arc;

use tokio::prelude::*;
use tokio::runtime::Runtime;
use tokio::timer::*;
use datasource::{DataSourceContainer, DataSource};
use datasource::yaml::YamlDataSource;

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
        Ok(source) => run_web_server(&mut runtime, Arc::new(source))
    };
}

fn run_web_server(runtime: &mut Runtime, container: Arc<DataSourceContainer>) {
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

    webserver::exec_webserver(container.clone())
}

struct RefreshFuture {
    datasource: Arc<DataSourceContainer>,
    interval: Interval
}

impl Stream for RefreshFuture {
    type Item = Arc<DataSourceContainer>;
    type Error = tokio::timer::Error;

    fn poll(&mut self) -> Result<Async<Option<<Self as Stream>::Item>>, <Self as Stream>::Error> {
        return match self.interval.poll() {
            Ok(Async::Ready(_)) => Ok(Async::Ready(Some(self.datasource.clone()))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(x) => Err(x)
        }
    }
}