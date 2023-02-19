use crate::storage::WorkerStore;
use gadget_lib::api::*;
use gadget_lib::prelude::{AliasRedirect, GadgetLibError, Redirect, RedirectModel};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use worker::kv::KvError;
use worker::*;

mod storage;
mod utils;

#[derive(Error, Debug)]
pub enum GadgetWorkerError {
    #[error(transparent)]
    GadgetLibError(#[from] GadgetLibError),
    #[error(transparent)]
    KvError(#[from] KvError),
    #[error(transparent)]
    WokerError(#[from] worker::Error),
}

impl From<GadgetWorkerError> for worker::Error {
    fn from(e: GadgetWorkerError) -> Self {
        worker::Error::RustError(e.to_string())
    }
}

#[derive(Serialize, Deserialize)]
struct StatusResponse {
    status: String,
}

type Result<T> = std::result::Result<T, GadgetWorkerError>;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> worker::Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let redirect_store = WorkerStore::new(&env).await?;
    let router = Router::with_data(redirect_store);

    router
        .put_async("/_api/redirect/*id", handle_update)
        .post_async("/_api/redirect", handle_create)
        .delete_async("/_api/redirect/*path", handle_any_delete)
        .get_async("/*path", handle_any_get)
        .run(req, env)
        .await
}

fn extract_param(ctx: &RouteContext<WorkerStore>, param: &str) -> Option<String> {
    let mut id = match ctx.param(param) {
        None => return None,
        Some(value) => value.to_owned(),
    };

    if id.starts_with('/') {
        id = id.replace('/', "");
    }

    if id.is_empty() {
        return None;
    }

    Some(id)
}

async fn handle_create(
    mut req: Request,
    ctx: RouteContext<WorkerStore>,
) -> worker::Result<Response> {
    let redirect: ApiRedirect = req.json().await?;
    let user = redirect
        .created_by
        .map(|x| x.username)
        .unwrap_or_else(|| "unknown".to_owned());

    match ctx
        .data
        .create_redirect(&redirect.alias, &redirect.destination, &user)
        .await
    {
        Ok(value) => Response::from_json(&value),
        Err(GadgetWorkerError::GadgetLibError(GadgetLibError::RedirectDoesNotExists(_))) => {
            worker::Response::error("Not found", 404)
        }
        Err(e) => worker::Response::error(e.to_string(), 501),
    }
}

async fn handle_update(
    mut req: Request,
    ctx: RouteContext<WorkerStore>,
) -> worker::Result<Response> {
    let id = match extract_param(&ctx, "id") {
        Some(id) => id,
        None => return Response::error("missing id", 400),
    };

    console_log!("Updating id {}", id);

    let redirect: UpdateRedirect = req.json().await?;
    let user = redirect
        .created_by
        .map(|x| x.username)
        .unwrap_or_else(|| "unknown".to_owned());

    match ctx
        .data
        .update_redirect(&id, &redirect.destination, &user)
        .await
    {
        Ok(value) => Response::from_json(&value),
        Err(GadgetWorkerError::GadgetLibError(GadgetLibError::RedirectDoesNotExists(_))) => {
            worker::Response::error("Not found", 404)
        }
        Err(e) => worker::Response::error(e.to_string(), 501),
    }
}

async fn handle_any_delete(
    _req: Request,
    ctx: RouteContext<WorkerStore>,
) -> worker::Result<Response> {
    let path = match extract_param(&ctx, "path") {
        Some(id) => id,
        None => return Response::error("missing path", 400),
    };

    let path = path.replace("%20", " ");
    let redirect_ref: Vec<&str> = path.split(' ').collect();
    let redirect_ref = match redirect_ref.first() {
        None => {
            return worker::Response::error("Not found", 404);
        }
        Some(value) => value,
    };

    match ctx.data.delete_redirect(redirect_ref).await {
        Ok(_) => Response::from_json(&StatusResponse {
            status: "Deleted".to_owned(),
        }),
        Err(GadgetWorkerError::GadgetLibError(GadgetLibError::RedirectDoesNotExists(_))) => {
            worker::Response::error("Not found", 404)
        }
        Err(e) => worker::Response::error(e.to_string(), 501),
    }
}

async fn handle_any_get(req: Request, ctx: RouteContext<WorkerStore>) -> worker::Result<Response> {
    if req.path() == "/_api/redirect" {
        let resp = ctx
            .data
            .get_all(0, 1000)
            .await
            .unwrap()
            .into_iter()
            .map(ApiRedirect::from)
            .collect();
        return Response::from_json(&RedirectList { redirects: resp });
    }

    if req.path().starts_with("/_api/redirect/") {
        match get_redirect(&req.path().replace("/_api/redirect/", ""), &ctx.data).await {
            Some(redirect) => return worker::Response::from_json(&ApiRedirect::from(redirect)),
            None => return worker::Response::error("Not found", 404),
        }
    }

    let path = req.path().replace('/', "").replace("%20", " ");
    let redirect_ref: Vec<&str> = path.split(' ').collect();
    let redirect_ref = match redirect_ref.first() {
        None => {
            return worker::Response::error("Not found", 404);
        }
        Some(value) => value,
    };

    console_debug!("Processing path {}", path);
    match get_redirect(redirect_ref, &ctx.data).await {
        Some(value) => {
            let redirect = AliasRedirect::from(value);
            worker::Response::redirect_with_status(
                worker::Url::parse(&redirect.get_destination(&path))?,
                307,
            )
        }
        None => worker::Response::error("Not found", 404),
    }
}

async fn get_redirect(path: &str, store: &WorkerStore) -> Option<RedirectModel> {
    console_debug!("Processing path {}", path);
    match store.get_redirect(path).await {
        Ok(Some(value)) => Some(value),
        _ => None,
    }
}
