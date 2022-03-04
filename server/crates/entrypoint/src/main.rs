use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    fmt::format::{Format, JsonFields, PrettyFields},
    layer::SubscriberExt,
    Registry,
};

use opentelemetry::{
    global,
    sdk::{
        propagation::TraceContextPropagator,
        trace::{self, IdGenerator, Sampler},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;

mod entrypoint;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    human_panic::setup_panic!();
    dotenv::dotenv().ok();

    let opt = entrypoint::Opts::parse();

    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(opt.runtime_args.otel_collector),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(IdGenerator::default())
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", opt.runtime_args.service_name),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap();

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let is_terminal = atty::is(atty::Stream::Stdout) && cfg!(debug_assertions);
    let pretty_logger = if is_terminal {
        Some(
            tracing_subscriber::fmt::layer()
                .event_format(Format::default().pretty())
                .fmt_fields(PrettyFields::new()),
        )
    } else {
        None
    };

    let json_logger = if !is_terminal {
        Some(
            tracing_subscriber::fmt::layer()
                .event_format(Format::default().json().flatten_event(true))
                .fmt_fields(JsonFields::new()),
        )
    } else {
        None
    };

    let subscriber = Registry::default()
        .with(LevelFilter::from(opt.logging_opts))
        .with(otel_layer)
        .with(json_logger)
        .with(pretty_logger);

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let _init = tracing_log::LogTracer::init().expect("logging to work correctly");

    entrypoint::execute_command(opt.sub_command).await
}
