#[cfg(feature = "postgres")]
#[macro_use] extern crate diesel;

use std::net::SocketAddr;
use std::sync::Arc;

use dotenv::dotenv;
use futures_util::join;
use warp::Filter;

use clap::{clap_app, crate_version};
use tracing::{error, level_filters::LevelFilter};
use tracing_timing::{Builder, Histogram};
use tracing_subscriber::{Registry, layer::SubscriberExt, fmt::format::FmtSpan};

use opentelemetry::{KeyValue, sdk::{trace::{self, IdGenerator, Sampler}, Resource}};

use crate::backend::BackendContainer;

use tracing_core::{Subscriber, Event};
use tracing_subscriber::fmt::{format::Format, FormatEvent, FormatFields, FmtContext};
use tracing_subscriber::registry::LookupSpan;

enum LoggingFormat {
    Json(Format<tracing_subscriber::fmt::format::Json>),
    Pretty(Format<tracing_subscriber::fmt::format::Pretty>),
}

impl Default for LoggingFormat {
    fn default() -> Self {
        if atty::is(atty::Stream::Stdout) {
            LoggingFormat::Pretty(tracing_subscriber::fmt::format().pretty())
        } else {
            LoggingFormat::Json(tracing_subscriber::fmt::format().json())
        }
    }
}

impl<S, N> FormatEvent<S, N> for LoggingFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        writer: &mut dyn std::fmt::Write,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        match self {
            LoggingFormat::Json(j) => j.format_event(ctx, writer, event),
            LoggingFormat::Pretty(p) => p.format_event(ctx, writer, event),
        }
    }
}

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
        (about: "Runs a go/ Links Server")
        (@group logging =>
            (@arg trace: --trace +global +multiple "Show trace logging")
            (@arg debug: -d --debug +global +multiple "Show debug logging")
            (@arg warn: -w --warn +global "Only display warning messages")
            (@arg error: --error +global "Only error output will be displayed")
        )
        (@arg ui_directory: --("ui-path") +takes_value env("UI_PATH") default_value("./public") "Where should the UI be served from?")
        (@arg listen_server: --listen +takes_value default_value("0.0.0.0:8080") "What port to should the main app listen on?")
        (@arg listen_metrics: --("listen-metrics") +takes_value default_value("0.0.0.0:8081") "Where should the metrics listen on?")
        (@arg DB_CONNECTION: --("database-url") +required +takes_value env("DATABASE_URL") "URL Database")
    )
    .get_matches();

    let level_filter = match (
        matches.is_present("error"),
        matches.is_present("warn"),
        matches.is_present("debug"),
        matches.is_present("trace"),
    ) {
        (true, _, _, _) => LevelFilter::ERROR,
        (false, true, _, _) => LevelFilter::WARN,
        (false, false, true, _) => LevelFilter::DEBUG,
        (false, false, false, true) => LevelFilter::TRACE,
        _ => LevelFilter::INFO,
    };

    let timing = Builder::default().layer_informed(|_s: &_, _e: &_| Histogram::new_with_max(1_000_000, 2).unwrap());
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(IdGenerator::default())
                .with_resource(Resource::new(vec![KeyValue::new("service.name", "gadget")]))
        )
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap();
        
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let console_output = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::CLOSE)
        .event_format(LoggingFormat::default());

    let subscriber = Registry::default()
        .with(level_filter)
        .with(otel_layer)
        .with(timing)
        .with(console_output);

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let backend_url = matches
        .value_of("DB_CONNECTION")
        .expect("To have a DB connection")
        .to_string();

    let backend = match BackendContainer::new(backend_url) {
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
        .with(warp::trace(|info| {
            // Create a span using tracing macros
            tracing::info_span!(
                "request",
                method = %info.method(),
                path = %info.path(),
            )
        }))
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

fn with_context(
    context: Arc<handlers::RequestContext>,
) -> impl Filter<Extract = (Arc<handlers::RequestContext>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || context.clone())
}

fn with_web_dir(
    web_dir: Arc<ui::WebDirectory>,
) -> impl Filter<Extract = (Arc<ui::WebDirectory>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || web_dir.clone())
}
