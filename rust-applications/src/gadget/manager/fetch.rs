use std::path::PathBuf;
use std::fs;
use std::time::Duration;

use super::config::*;
use crate::config::read_config;
use crate::webserver::config::ConfigRoot;

pub fn run_fetcher(config_path: PathBuf, dest: PathBuf) {
    let config: ManagerConfig = match read_config(config_path.clone()) {
        Ok(config_root) => config_root,
        Err(err) => panic!("{}", err)
    };

    let sleep_duration = Duration::from_secs(config.frequency as u64);

    loop {
        do_update(&config, &dest);
        std::thread::sleep(sleep_duration);
    }
}

fn do_update(config: &ManagerConfig, dest: &PathBuf) {
    debug!("Fetching config...");

    let fetched_text = match &config.poller {
        Poller::Web(fetcher) => fetcher.do_fetch()
    };

    let fetched_text = match fetched_text {
        Some(text) => text,
        None => return
    };

    if let Err(e) = serde_yaml::from_str::<ConfigRoot>(&fetched_text) {
        warn!("🛑 Unable to parse pulled config: {}", e);
        return;
    }

    match fs::write(dest, fetched_text) {
        Ok(_) => info!("✅ Configs written"),
        Err(e) => warn!("Unable to write to config file: {}", e)
    }
}