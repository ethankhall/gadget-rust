use clap::{ArgGroup, Args, Parser, Subcommand};
use futures_util::join;
use gadget_api::prelude::{make_api_filters, metrics_endpoint, SharedData};
use gadget_backend::DefaultBackend;
use std::sync::Arc;
use tracing::error;
use tracing::level_filters::LevelFilter;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
pub struct Opts {
    #[clap(subcommand)]
    pub sub_command: MainOperation,

    #[clap(flatten)]
    pub logging_opts: LoggingOpts,

    #[clap(flatten)]
    pub runtime_args: RuntimeArgs,
}

#[derive(Subcommand, Debug)]
pub enum MainOperation {
    /// Run the web server
    #[clap(name = "serve")]
    RunWebServer(RunWebServerArgs),

    /// Run the DB Migration
    #[clap(name = "run-db-migrations")]
    DatabaseMigration(RunDatabaseMigrationsArgs),
}

#[derive(Args, Debug)]
pub struct RunWebServerArgs {
    #[clap(flatten)]
    pub db_args: DatabaseArguments,

    /// Address to expose the main API on
    #[clap(
        long = "server-address",
        env = "SERVER_ADDRESS",
        default_value("127.0.0.1:3030")
    )]
    pub server_address: String,

    /// Address to expose the main API on
    #[clap(
        long = "metrics-address",
        env = "METRICS_ADDRESS",
        default_value("127.0.0.1:3031")
    )]
    pub metrics_address: String,

    #[clap(
        long = "ui-location",
        env = "UI_LOCATION",
        default_value("http:///127.0.0.1:3032")
    )]
    pub ui_location: String,
}

#[derive(Args, Debug)]
pub struct RunDatabaseMigrationsArgs {
    #[clap(flatten)]
    pub db_args: DatabaseArguments,
}

#[derive(Args, Debug)]
pub struct DatabaseArguments {
    /// Database Connection String
    #[clap(long = "database-url", env = "DATABASE_URL")]
    pub db_connection_string: String,
}

#[derive(Args, Debug)]
pub struct RuntimeArgs {
    /// The URL to publish metrics to.
    #[clap(
        long = "open-telem-collector",
        env = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
        default_value("http://localhost:4317")
    )]
    pub otel_collector: String,

    /// The service name to be tagged with all telemitry data.
    #[clap(
        long = "metric-service-name",
        env = "METRIC_SERVICE_NAME",
        default_value("gadget")
    )]
    pub service_name: String,
}

#[derive(Parser, Debug)]
#[clap(group = ArgGroup::new("logging"))]
pub struct LoggingOpts {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences), global(true), group = "logging")]
    pub debug: u64,

    /// Enable warn logging
    #[clap(short, long, global(true), group = "logging")]
    pub warn: bool,

    /// Disable everything but error logging
    #[clap(short, long, global(true), group = "logging")]
    pub error: bool,
}

impl From<LoggingOpts> for LevelFilter {
    fn from(opts: LoggingOpts) -> Self {
        if opts.error {
            LevelFilter::ERROR
        } else if opts.warn {
            LevelFilter::WARN
        } else if opts.debug == 0 {
            LevelFilter::INFO
        } else if opts.debug == 1 {
            LevelFilter::DEBUG
        } else {
            LevelFilter::TRACE
        }
    }
}

pub async fn execute_command(sub_command: MainOperation) -> Result<(), anyhow::Error> {
    match sub_command {
        MainOperation::RunWebServer(args) => run_webserver(args).await,
        MainOperation::DatabaseMigration(args) => run_db_migration(args).await,
    }
}

async fn run_webserver(args: RunWebServerArgs) -> Result<(), anyhow::Error> {
    use std::net::SocketAddr;
    use warp::Filter;

    let ui_location = match url::Url::parse(&args.ui_location) {
        Ok(url) => url,
        Err(e) => {
            error!(
                "Unable to parse {} into URL, as it is not a URL. Error: `{}`",
                args.ui_location,
                e.to_string()
            );
            return Err(e.into());
        }
    };

    let backend = DefaultBackend::new(&args.db_args.db_connection_string).await?;
    let backend = Arc::new(SharedData {
        ui_location,
        backend,
    });

    let filters = make_api_filters(backend).await;

    let api_addr: SocketAddr = args.server_address.parse()?;
    let api_server = warp::serve(filters).run(api_addr);

    let metrics_server = warp::path("metrics")
        .map(metrics_endpoint)
        .or(warp::path("status").map(|| "OK"))
        .with(warp::trace::request());

    let metrics_addr: SocketAddr = args.metrics_address.parse()?;
    let metrics_server = warp::serve(metrics_server).run(metrics_addr);

    let (_main, _metrics) = join!(api_server, metrics_server);

    Ok(())
}

async fn run_db_migration(args: RunDatabaseMigrationsArgs) -> Result<(), anyhow::Error> {
    gadget_migration::run_db_migration(&args.db_args.db_connection_string).await?;

    Ok(())
}
