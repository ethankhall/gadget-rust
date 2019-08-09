use std::path::PathBuf;

use hotwatch::{Event, Hotwatch};

use super::config::*;
use crate::config::read_config;

pub fn run_syncer(config_path: PathBuf, target: PathBuf) {
    let config_root: PusherConfig =
        read_config(config_path.clone()).expect("Unable to read initial config!");

    let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
    {
        hotwatch
            .watch(target.clone(), move |event: Event| {
                if let Event::Write(path) = event {
                    match &config_root.pusher {
                        Pusher::Azure(azure) => azure.push_blob(path.as_path()),
                        Pusher::S3(s3) => s3.push_blob(path.as_path()),
                    }

                    info!("File {:?} was uploaded", path);
                }
            })
            .expect("Failed to watch file!");
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(60 * 60));
    }
}
