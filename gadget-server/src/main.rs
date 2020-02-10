#[cfg(feature = "postgres")]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prometheus;

use std::net::SocketAddr;
use std::sync::Arc;

use dotenv::dotenv;
use flexi_logger::{LevelFilter, LogSpecBuilder, Logger};
use futures_util::join;
use warp::Filter;

use crate::backend::{make_backend, Backend};

#[macro_export]
macro_rules! s {
    ($x:expr) => {
        $x.to_string()
    };
}

mod admin;
mod backend;
mod handlers;
mod redirect;
mod ui;

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    dotenv().ok();

    let matches = clap_app!(gadget =>
        (version: crate_version!())
        (about: "Creates a Slack bot with PagerDuty")
        (@group logging =>
            (@arg debug: -v --verbose +global "Increasing verbosity")
            (@arg warn: -w --warn +global "Only display warning messages")
            (@arg quite: -q --quite +global "Only error output will be displayed")
        )
        (@arg ui_directory: --("ui-path") +takes_value env("UI_PATH") default_value("./public") "Where should the UI be served from?")
        (@arg listen_server: --listen +takes_value default_value("0.0.0.0:8080") "What port to should the main app listen on?")
        (@arg listen_metrics: --("listen-metrics") +takes_value default_value("0.0.0.0:8081") "Where should the metrics listen on?")
        (@arg DB_CONNECTION: --("database-url") +required +takes_value env("DATABASE_URL") "URL Database")
    )
    .get_matches();

    let level_filter = match (
        matches.is_present("quite"),
        matches.is_present("warn"),
        matches.is_present("debug"),
    ) {
        (true, _, _) => LevelFilter::Error,
        (false, true, _) => LevelFilter::Warn,
        (false, false, false) => LevelFilter::Info,
        (false, false, true) => LevelFilter::Debug,
    };

    let mut builder = LogSpecBuilder::new(); // default is LevelFilter::Off
    builder.default(level_filter);

    Logger::with(builder.build())
        .format(custom_log_format)
        .start()
        .unwrap();

    let backend_url = matches
        .value_of("DB_CONNECTION")
        .expect("To have a DB connection")
        .to_string();

    let backend = match make_backend(backend_url) {
        Ok(backend) => backend,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };

    let backend = handlers::RequestContext::new(backend);

    let backend = Arc::new(backend);

    let ui_root_dir = matches.value_of("ui_directory").expect("To have UI Path");
    let web_dir = match ui::WebDirectory::new(ui_root_dir.to_string()) {
        Some(x) => x,
        None => return Err("Unable to access UI directory"),
    };
    let web_dir = Arc::new(web_dir);

    let main_server = warp::path!("favicon.ico")
        .and_then(handlers::favicon)
        .or(warp::path!("_gadget" / "api" / "redirect")
            .and(warp::get())
            .and(with_context(backend.clone()))
            .and_then(handlers::list_redirects))
        .or(warp::path!("_gadget" / "api" / "redirect")
            .and(warp::post())
            .and(handlers::json_body())
            .and(handlers::extract_user())
            .and(with_context(backend.clone()))
            .and_then(handlers::new_redirect_json))
        .or(warp::path!("_gadget" / "api" / "redirect" / String)
            .and(warp::get())
            .and(with_context(backend.clone()))
            .and_then(handlers::get_redirect))
        .or(warp::path!("_gadget" / "api" / "redirect" / String)
            .and(warp::delete())
            .and(with_context(backend.clone()))
            .and_then(handlers::delete_redirect))
        .or(warp::path!("_gadget" / "api" / "redirect" / String)
            .and(warp::put())
            .and(handlers::json_body())
            .and(handlers::extract_user())
            .and(with_context(backend.clone()))
            .and_then(handlers::update_redirect))
        .or(warp::path("_gadget")
            .and(warp::path("ui"))
            .and(warp::path::tail())
            .and(warp::get())
            .and(with_web_dir(web_dir))
            .and_then(ui::serve_embedded))
        .or(warp::get()
            .and(warp::path::tail())
            .and(with_context(backend.clone()))
            .and_then(handlers::find_redirect))
        .or(warp::any().map(|| {
            handlers::ResponseMessage::from("not found")
                .into_raw_response(warp::http::StatusCode::NOT_FOUND)
        }))
        .with(warp::log("api"))
        .with(warp::log::custom(admin::track_status));

    let listen_server: SocketAddr = matches
        .value_of("listen_server")
        .expect("To be able to get listen_server address")
        .parse()
        .expect("Unable to parse listen_server");

    let main_server = warp::serve(main_server).run(listen_server);

    let admin_server = warp::path("metrics")
        .map(admin::metrics_endpoint)
        .or(warp::path!("status").map(|| {
            handlers::ResponseMessage::from("OK").into_raw_response(warp::http::StatusCode::OK)
        }));

    let listen_metrics: SocketAddr = matches
        .value_of("listen_metrics")
        .expect("To be able to get listen_metrics address")
        .parse()
        .expect("Unable to parse listen_metrics");

    let admin_server = warp::serve(admin_server).run(listen_metrics);

    let (_main, _admin) = join!(main_server, admin_server);

    Ok(())
}

fn with_context<T>(
    context: Arc<handlers::RequestContext<T>>,
) -> impl Filter<Extract = (Arc<handlers::RequestContext<T>>,), Error = std::convert::Infallible> + Clone where T: Backend {
    warp::any().map(move || context.clone())
}

fn with_web_dir(
    web_dir: Arc<ui::WebDirectory>,
) -> impl Filter<Extract = (Arc<ui::WebDirectory>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || web_dir.clone())
}

fn custom_log_format(
    w: &mut dyn std::io::Write,
    now: &mut flexi_logger::DeferredNow,
    record: &flexi_logger::Record,
) -> Result<(), std::io::Error> {
    use flexi_logger::style;
    use std::thread;

    let level = record.level();
    write!(
        w,
        "[{}] T[{:?}] {} [{}:{}] {}",
        style(level, now.now().format("%Y-%m-%d %H:%M:%S%.6f %:z")),
        style(level, thread::current().name().unwrap_or("<unnamed>")),
        style(level, level),
        record.module_path().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
        style(level, &record.args())
    )
}
