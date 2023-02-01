use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
use dotenv::dotenv;
use gadget_lib::{api::{RedirectList, ApiRedirect, UpdateRedirect}, AliasRedirect, Redirect, prelude::RedirectModel};
use human_panic::setup_panic;
use log::{debug, error, trace};
use reqwest::{Method};
use std::path::PathBuf;
use thiserror::Error as ThisError;
use url::Url;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: CommandOptions,

    #[clap(flatten)]
    api_options: ApiOptions,
}

#[derive(Args, Debug)]
struct ApiOptions {
    /// Server name to communicate with with
    #[clap(
        long,
        value_parser,
        default_value = "https://api.gto.cx",
        env("GADGET_API_SERVER")
    )]
    server_name: String,

    /// Authorization settings for the API
    #[clap(flatten)]
    auth: ApiAuth,
}

impl ApiOptions {
    fn create_client(&self) -> Result<reqwest::Client, CliError> {
        let mut builder = reqwest::ClientBuilder::new();
        if self.auth.auth_type == AuthType::MutualAuth {
            if let Some(ca_path) = self.auth.mtls_ca.as_ref() {
                if !ca_path.exists() {
                    return Err(CliError::CertificateError(ca_path.display().to_string()));
                }

                debug!("Loading cert from {:?}", ca_path);
                let ca_bytes = std::fs::read(ca_path)?;
                let cert = reqwest::Certificate::from_pem(&ca_bytes)?;
                debug!("Loaded CA");

                builder = builder.add_root_certificate(cert);

                debug!("Added ROOT CA")
            }

            let cert_path = self.auth.mtls_cert.as_ref().unwrap();
            if !cert_path.exists() {
                return Err(CliError::CertificateError(cert_path.display().to_string()));
            }
            debug!("Loading identity from {:?}", cert_path);
            let cert_bytes = std::fs::read(cert_path)?;
            let identity = reqwest::Identity::from_pem(&cert_bytes)?;
            debug!("Loaded Identity");

            builder = builder.use_rustls_tls().identity(identity).https_only(true);
        }

        trace!("builder {:?}", builder);

        Ok(builder.build()?)
    }

    async fn make_request<T, R>(
        &self,
        url_path: &str,
        method: Method,
        body: Option<&T>,
    ) -> Result<R, CliError>
    where
        T: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let client = self.create_client()?;
        let url: Url = format!("{}{}", self.server_name, url_path).parse()?;
        debug!("Request URL is {}", url);

        let mut builder = client
            .request(method.clone(), url.clone());

        if let Some(real_body) = body {
            builder = builder.json(real_body);
        }

        let response = builder.send().await?;

        trace!("response {:?}", response);
        let status = response.status();
        let bytes = response.bytes().await?;

        debug!("un-seralized body {:?}", bytes);
        if !status.is_success() {
            return Err(CliError::ApiError {
                method: method.to_string(),
                url,
                status: status.to_string(),
            });
        }

        let response_body = String::from_utf8(bytes.to_vec())?;
        let body: R = serde_json::from_str(&response_body)?;

        Ok(body)
    }
}

#[derive(Args, Debug)]
#[clap(group(
    ArgGroup::new("mtls").multiple(true)
))]
struct ApiAuth {
    /// No authentication
    #[clap(
        name = "auth",
        long,
        global(true),
        value_parser,
        default_value = "none"
    )]
    auth_type: AuthType,

    /// Path to x509 PEM file containing BOTH a certificate and key in PKCS#8 format.
    #[clap(name = "cert", long, global(true), value_parser, value_hint = clap::ValueHint::DirPath, required_if_eq("auth", "mtls"))]
    mtls_cert: Option<PathBuf>,

    /// Path to x509 CA if not a trusted root
    #[clap(name = "ca", long, global(true), value_parser, value_hint = clap::ValueHint::DirPath, requires="cert")]
    mtls_ca: Option<PathBuf>,
}

#[derive(ValueEnum, PartialEq, Debug, Clone)]
enum AuthType {
    None,
    #[clap(name = "mtls", alias = "x509")]
    MutualAuth,
}

#[derive(Subcommand, Debug)]
enum CommandOptions {
    /// List all the redirects avaliable
    List,
    /// Get a single redirect
    Get(GetArgs),
    /// Delete a redirect
    Delete(DeleteArgs),
    /// Update a redirect
    Update(TupleArgs),
    /// Create a redirect
    Create(TupleArgs),
}

#[derive(Args, Debug)]
struct TupleArgs {
    /// Name of the redirect
    #[clap(long, value_parser)]
    alias: String,

    /// Where the redirect will be send to
    #[clap(long, value_parser)]
    destination: String,
}

#[derive(Args, Debug)]
struct GetArgs {
    /// Name of the redirect
    #[clap(value_parser)]
    alias: String,

    /// When specified, the redirect will be evaluated
    #[clap(long, value_parser)]
    options: Vec<String>
}

#[derive(Args, Debug)]
struct DeleteArgs {
    /// Name of the redirect
    #[clap(value_parser)]
    alias: String,
}


#[derive(ThisError, Debug)]
pub enum CliError {
    #[error(transparent)]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("Certificate {0} does not exist or cannot be read")]
    CertificateError(String),
    #[error(transparent)]
    ReqwestError {
        #[from]
        source: reqwest::Error,
    },
    #[error(transparent)]
    ParseError(#[from] url::ParseError),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error("Request {method} {url} failed with {status}. See debug logs for actual response")]
    ApiError {
        method: String,
        url: Url,
        status: String,
    },
    #[error(transparent)]
    UnicodeError(#[from] std::string::FromUtf8Error),
}

fn main() {
    setup_panic!();
    dotenv().ok();
    env_logger::init();

    let args = Cli::parse();
    if let Err(e) = run_command(args) {
        error!("{}", e.to_string());
    }
}

#[tokio::main]
async fn run_command(opts: Cli) -> Result<(), CliError> {
    match opts.command {
        CommandOptions::List => run_list(&opts.api_options).await?,
        CommandOptions::Get(args) => run_get(&args, &opts.api_options).await?,
        CommandOptions::Create(args) => run_create(&args, &opts.api_options).await?,
        CommandOptions::Delete(args) => run_delete(&args, &opts.api_options).await?,
        CommandOptions::Update(args) => run_update(&args, &opts.api_options).await?,
    }

    Ok(())
}

async fn run_list(api_opts: &ApiOptions) -> Result<(), CliError> {
    let body: RedirectList = api_opts
        .make_request::<(), _>("/_api/redirect", Method::GET, None)
        .await?;
    crate::output::show_redirects(body.redirects);

    Ok(())
}

async fn run_get(args: &GetArgs, api_opts: &ApiOptions) -> Result<(), CliError> {
    let body: ApiRedirect = api_opts
        .make_request::<(), _>(&format!("/_api/redirect/{}", args.alias), Method::GET, None)
        .await?;

    println!("Redirect target: {}", body.destination);
    let redirect = AliasRedirect::from(body);

    for test_dest in &args.options {
        println!("'{}/{}' will redirect to {}", &args.alias, &test_dest, redirect.evaluate(&test_dest));
    }
    Ok(())
}

async fn run_delete(args: &DeleteArgs, api_opts: &ApiOptions) -> Result<(), CliError> {
    api_opts
        .make_request::<(), _>(&format!("/_api/redirect/{}", args.alias), Method::DELETE, None)
        .await?;
    Ok(())
}

async fn run_create(args: &TupleArgs, api_opts: &ApiOptions) -> Result<(), CliError> {
    let redirect = ApiRedirect {
        alias: args.alias.clone(), 
        destination: args.destination.clone(),
        created_by: None
    };

    let body: RedirectModel = api_opts
        .make_request("/_api/redirect", Method::POST, Some(&redirect))
        .await?;

    println!("Created: {}", body.alias);
    Ok(())
}

async fn run_update(args: &TupleArgs, api_opts: &ApiOptions) -> Result<(), CliError> {
    let redirect = UpdateRedirect {
        destination: args.destination.clone(),
        created_by: None
    };

    let body: ApiRedirect = api_opts
        .make_request(&format!("/_api/redirect/{}", args.alias), Method::PUT, Some(&redirect))
        .await?;

    println!("Created: {}", body.alias);
    Ok(())
}

mod output {
    use gadget_lib::api::ApiRedirect;
    use tabled::{Table, Tabled};

    #[derive(Tabled)]
    struct TableRedirect {
        #[tabled(rename = "Alias")]
        alias: String,
        #[tabled(rename = "Destination")]
        destination: String,
        #[tabled(rename = "Created By")]
        created_by: String,
    }

    impl From<ApiRedirect> for TableRedirect {
        fn from(api: ApiRedirect) -> Self {
            Self {
                alias: api.alias.clone(),
                destination: api.destination.clone(),
                created_by: api
                    .created_by
                    .as_ref()
                    .map(|x| x.username.clone())
                    .unwrap_or("".to_string()),
            }
        }
    }

    pub fn show_redirects(input: Vec<ApiRedirect>) {
        let table_redirect: Vec<TableRedirect> =
            input.into_iter().map(TableRedirect::from).collect();
        let table = Table::new(table_redirect).to_string();
        println!("{}", table);
    }
}
