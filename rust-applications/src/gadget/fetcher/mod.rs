use std::path::PathBuf;
use std::fs;
use std::time::Duration;

use reqwest::Client;

use crate::config::{*, fetch::FetchConfig};

pub fn run_fetcher(config_path: PathBuf, dest: PathBuf) {
    let config: FetchConfig = match read_config(config_path.clone()) {
        Ok(config_root) => config_root,
        Err(err) => panic!("{}", err)
    };

    let client = Client::new();
    let sleep_duration = Duration::from_secs(config.frequency as u64);

    loop {
        do_update(&client, &config, &dest);
        std::thread::sleep(sleep_duration);
    }
}

fn do_update(client: &Client, config: &FetchConfig, dest: &PathBuf) {
    debug!("Fetching config...");
    
    let url = &config.fetch_url;
    let mut resp_body = match client.get(url).query(&[("id", config.fetch_id.clone())]).send() {
        Ok(resp) => resp,
        Err(e) => {
            warn!("📮 Unable to pull configs: {}", e);
            return;
        }
    };

    let fetched_text = match resp_body.text() {
        Ok(text) => text,
        Err(e) => {
            warn!("⚠️ Unable to get text: {}", e);
            return;
        }
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