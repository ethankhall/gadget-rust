pub mod config;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use chrono::prelude::*;
use hotwatch::{Event, Hotwatch};
use hyper::rt::{self, Future};
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};

use self::config::*;
use crate::config::read_config;

pub fn run_webserver(bind_addr: SocketAddr, config_path: PathBuf) {
    let config_root: ConfigRoot = match read_config(config_path.clone()) {
        Ok(config_root) => config_root,
        Err(err) => panic!("{}", err),
    };

    info!("📚 Config read, ready to process!");

    let compiled_config = config_root.compile();
    debug!("Config: {:?}", compiled_config);
    let shared_config = Arc::new(RwLock::new(compiled_config));

    let mut hotwatch = Hotwatch::new().expect("Hotwatch failed to initialize.");
    {
        let hotwatch_config = shared_config.clone();
        hotwatch
            .watch(config_path.clone(), move |event: Event| {
                let config: Arc<RwLock<CompiledConfigs>> = hotwatch_config.clone();
                if let Event::Write(path) = event {
                    match read_config::<ConfigRoot>(path) {
                        Ok(config_root) => {
                            *config.write().unwrap() = config_root.compile();
                            info!("Configs were reloaded at {}.", Local::now())
                        }
                        Err(err) => {
                            warn!("Unable to open updated config ({}), ignoring...", err);
                        }
                    };
                }
            })
            .expect("Failed to watch file!");
    }

    let new_service = move || {
        let config: Arc<RwLock<CompiledConfigs>> = shared_config.clone();
        service_fn_ok(move |req| {
            let lock = config.read().unwrap();
            let path = req.uri().path();
            debug!("Request for {:?}", path);
            let redirect = lock.find_redirect(path);
            Response::builder()
                .status(307)
                .header(hyper::header::LOCATION, redirect)
                .body(Body::empty())
                .unwrap()
        })
    };

    let server = Server::bind(&bind_addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", bind_addr);

    std::thread::spawn(move || {
        rt::run(server);
        println!("Exiting server")
    });

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
