#[macro_use]
extern crate diesel;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use flexi_logger::{colored_with_thread, LevelFilter, LogSpecBuilder, Logger};

#[macro_export]
macro_rules! s {
    ($x:expr) => {
        $x.to_string()
    };
}

mod backend;
mod handlers;
mod redirect;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let matches = clap_app!(gadget =>
        (version: crate_version!())
        (about: "Creates a Slack bot with PagerDuty")
        (@group logging =>
            (@arg debug: -v --verbose +global "Increasing verbosity")
            (@arg warn: -w --warn +global "Only display warning messages")
            (@arg quite: -q --quite +global "Only error output will be displayed")
        )
        (@arg listen_server: --listen +takes_value default_value("0.0.0.0:8080") "What port to listen on.")
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
        .format(colored_with_thread)
        .start()
        .unwrap();

    let backend = handlers::Context::new(
        matches
            .value_of("DB_CONNECTION")
            .expect("To have a DB connection"),
    );

    HttpServer::new(move || {
        App::new()
            .service(web::resource("favicon.ico").to(handlers::favicon))
            .route(
                "/_gadget/redirect",
                web::post().to(handlers::new_redirect_json),
            )
            .route(
                "/_gadget/redirect/{id}",
                web::delete().to(handlers::delete_redirect),
            )
            .route(
                "/_gadget/redirect/{id}",
                web::put().to(handlers::update_redirect),
            )
            .route("/{path:.*}", web::get().to(handlers::find_redirect))
            .data(backend.clone())
            .wrap(middleware::Logger::default())
            .default_service(web::to(|| async { "404" }))
    })
    .bind(
        matches
            .value_of("listen_server")
            .expect("To be able to get listen address"),
    )?
    .start()
    .await
}
