#[cfg(feature = "postgres")]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prometheus;
use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use flexi_logger::{LevelFilter, LogSpecBuilder, Logger};
use futures_util::join;

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

#[actix_rt::main]
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

    let backend = handlers::Context::new(
        matches
            .value_of("DB_CONNECTION")
            .expect("To have a DB connection"),
    );

    let ui_root_dir = matches.value_of("ui_directory").expect("To have UI Path");
    let web_dir = match ui::WebDirectory::new(ui_root_dir.to_string()) {
        Some(x) => x,
        None => {
            return Err("Unable to access UI directory")
        }
    };

    let main_server = HttpServer::new(move || {
        App::new()
            .wrap(admin::Metrics::default())
            .service(web::resource("favicon.ico").to(handlers::favicon))
            .service(
                web::resource("/_gadget/api/redirect")
                    .name("make_redirect")
                    .route(web::post().to(handlers::new_redirect_json))
                    .route(web::get().to(handlers::list_redirects)),
            )
            .service(
                web::resource("/_gadget/api/redirect/{id}")
                    .name("change_redirect")
                    .route(web::delete().to(handlers::delete_redirect))
                    .route(web::put().to(handlers::update_redirect))
                    .route(web::get().to(handlers::get_redirect)),
            )
            .route("/_gadget/ui", web::get().to(ui::serve_embedded))
            .route("/_gadget/ui/{filename:.*}", web::get().to(ui::serve_embedded))
            .route("/_gadget/.*", web::to(|| async { "404" }))
            .route("/{path:.*}", web::get().to(handlers::find_redirect))
            .data(backend.clone())
            .data(web_dir.clone())
            .default_service(web::to(|| async { "404" }))
    })
    .bind(
        matches
            .value_of("listen_server")
            .expect("To be able to get listen address"),
    )
    .expect("to be able to bind to http address")
    .run();

    let admin_server = HttpServer::new(|| {
        App::new()
            .route("/metrics", web::get().to(admin::metrics_endpoint))
            .route("/status", web::get().to(|| HttpResponse::from("OK")))
            .default_service(web::to(|| async { "404" }))
    })
    .bind(
        matches
            .value_of("listen_metrics")
            .expect("To be able to get listen address"),
    )
    .expect("to be able to bind to metrics address")
    .run();

    let (_main, _admin) = join!(main_server, admin_server);
    Ok(())
}

fn custom_log_format(
    w: &mut dyn std::io::Write,
    now: &mut  flexi_logger::DeferredNow,
    record: & flexi_logger::Record,
) -> Result<(), std::io::Error> {
    use std::thread;
    use flexi_logger::style;

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