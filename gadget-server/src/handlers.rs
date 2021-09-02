use std::convert::Infallible;
use std::sync::Arc;
use tracing::{instrument, debug, error, info, trace, warn};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;
use warp::{
    http::header::LOCATION,
    http::{HeaderMap, HeaderValue, StatusCode},
    reply::Reply,
    Filter,
};

use crate::backend::{BackendContainer, RedirectModel, RowChange};
use crate::redirect::{AliasRedirect, Redirect};

#[derive(Clone)]
pub struct RequestContext {
    backend: Arc<BackendContainer>,
}

unsafe impl std::marker::Send for RequestContext {}

unsafe impl std::marker::Sync for RequestContext {}

impl RequestContext {
    pub fn new(backend: BackendContainer) -> Self {
        RequestContext {
            backend: Arc::new(backend),
        }
    }
}

#[derive(Serialize)]
pub struct ResponseMessage {
    message: String,
}

impl From<&str> for ResponseMessage {
    fn from(message: &str) -> ResponseMessage {
        ResponseMessage {
            message: message.to_string(),
        }
    }
}

impl From<String> for ResponseMessage {
    fn from(message: String) -> ResponseMessage {
        ResponseMessage { message }
    }
}

impl ResponseMessage {
    pub fn into_response(
        self,
        status_code: StatusCode,
    ) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
        Ok(self.into_raw_response(status_code))
    }

    pub fn into_raw_response(
        self,
        status_code: StatusCode,
    ) -> warp::reply::WithStatus<warp::reply::Json> {
        warp::reply::with_status(warp::reply::json(&self), status_code)
    }
}

#[derive(Deserialize, Debug)]
pub struct NewRedirect {
    alias: String,
    destination: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateRedirect {
    destination: String,
}

#[derive(Serialize)]
pub struct ApiRedirect {
    id: String,
    alias: String,
    destination: String,
    created_by: Option<String>,
}

impl Into<ApiRedirect> for RedirectModel {
    fn into(self) -> ApiRedirect {
        ApiRedirect {
            id: self.public_ref,
            alias: self.alias,
            destination: self.destination,
            created_by: self.created_by,
        }
    }
}

#[derive(Serialize)]
struct RedirectList {
    redirects: Vec<ApiRedirect>,
}

pub async fn favicon() -> Result<impl warp::Reply, Infallible> {
    Ok(StatusCode::NOT_FOUND)
}

#[instrument(skip(context))]
pub async fn delete_redirect(
    path: String,
    context: Arc<RequestContext>,
) -> Result<impl warp::Reply, Infallible> {
    let resp = context.backend.delete_redirect(&path);

    match resp {
        RowChange::NotFound => {
            ResponseMessage::from("not found").into_response(StatusCode::NOT_FOUND)
        }
        RowChange::Value(_) => ResponseMessage::from("ok").into_response(StatusCode::OK),
        RowChange::Err(e) => {
            error!("Unable to update redirect: {:?}", e);
            ResponseMessage::from(format!("Unexpected error: {:?}", e))
                .into_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub fn json_body<T: DeserializeOwned + Send>(
) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[instrument(skip(context))]
pub async fn new_redirect_json(
    info: NewRedirect,
    user: UserDetails,
    context: Arc<RequestContext>,
) -> Result<impl warp::Reply, Infallible> {
    if !is_destination_url(&info.destination) {
        debug!("Destination wasn't URL {:?}", &info.destination);
        return ResponseMessage::from(format!("{:?} isn't a valid URL", &info.destination))
            .into_response(StatusCode::BAD_REQUEST);
    }

    info!("Creating redirect {} => {}", info.alias, info.destination);
    match context
        .backend
        .create_redirect(&info.alias, &info.destination, &user.username)
    {
        RowChange::Value(result) => {
            let api_model: ApiRedirect = result.into();
            Ok(warp::reply::with_status(
                warp::reply::json(&api_model),
                StatusCode::CREATED,
            ))
        }
        RowChange::Err(e) => {
            warn!("Unable to create redirect: {:?}", e);
            ResponseMessage::from(format!("Unable to create redirect: {:?}", e))
                .into_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
        RowChange::NotFound => {
            warn!("Unable to create redirect");
            ResponseMessage::from("Unable to create redirect")
                .into_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[instrument(skip(context))]
pub async fn update_redirect(
    info: String,
    dest: UpdateRedirect,
    user: UserDetails,
    context: Arc<RequestContext>,
) -> Result<impl warp::Reply, Infallible> {
    if !is_destination_url(&dest.destination) {
        debug!("Destination wasn't URL {:?}", &dest.destination);
        return ResponseMessage::from(format!("{:?} isn't a valid URL", &dest.destination))
            .into_response(StatusCode::BAD_REQUEST);
    }

    let resp = context
        .backend
        .update_redirect(&info, &dest.destination, &user.username);

    match resp {
        RowChange::NotFound => {
            ResponseMessage::from("not found").into_response(StatusCode::NOT_FOUND)
        }
        RowChange::Value(_) => ResponseMessage::from("ok").into_response(StatusCode::OK),
        RowChange::Err(e) => {
            error!("Unable to update redirect: {:?}", e);
            ResponseMessage::from(format!("Unexpected error: {:?}", e))
                .into_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[instrument(skip(context))]
pub async fn list_redirects(context: Arc<RequestContext>) -> Result<impl warp::Reply, Infallible> {
    let resp = match context.backend.get_all(0, 10000) {
        RowChange::Value(v) => {
            let data: Vec<ApiRedirect> = v.into_iter().map(|x| x.into()).collect();
            RedirectList { redirects: data }
        }
        RowChange::NotFound => RedirectList { redirects: vec![] },
        RowChange::Err(e) => {
            warn!("Unable to get redirect: {:?}", e);
            return ResponseMessage::from("Unable to get redirect")
                .into_response(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&resp),
        StatusCode::OK,
    ))
}

fn is_destination_url(path: &str) -> bool {
    Url::parse(&path).is_ok()
}

#[instrument(skip(context))]
pub async fn get_redirect(
    info: String,
    context: Arc<RequestContext>,
) -> Result<impl warp::Reply, Infallible> {
    match context.backend.get_redirect(&info) {
        RowChange::Value(value) => {
            let redirect: ApiRedirect = value.into();
            Ok(warp::reply::with_status(
                warp::reply::json(&redirect),
                StatusCode::OK,
            ))
        }
        RowChange::NotFound => {
            ResponseMessage::from("Unable to get redirect").into_response(StatusCode::NOT_FOUND)
        }
        RowChange::Err(e) => {
            warn!("Unable to get redirect: {:?}", e);
            ResponseMessage::from("Unable to get redirect")
                .into_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tracing::instrument(skip(context))]
pub async fn find_redirect(
    path: warp::filters::path::Tail,
    context: Arc<RequestContext>,
) -> Result<warp::reply::Response, Infallible> {
    let info = path.as_str().replace("%20", " ");

    let redirect_ref: Vec<&str> = info.split(' ').collect();
    let redirect_ref = match redirect_ref.first() {
        None => {
            return Ok(warp::http::Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, "/_gadget/ui")
                .body(hyper::Body::empty())
                .unwrap());
        }
        Some(value) => value,
    };

    match context.backend.get_redirect(redirect_ref) {
        RowChange::Value(value) => {
            let redirect = AliasRedirect::from(value);
            Ok(warp::http::Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, redirect.get_destination(&info))
                .body(hyper::Body::empty())
                .unwrap())
        }
        RowChange::NotFound => Ok(warp::http::Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, format!("/_gadget/ui?search={}", &info))
            .body(hyper::Body::empty())
            .unwrap()),
        RowChange::Err(e) => {
            warn!("Unable to get redirect: {:?}", e);
            ResponseMessage::from("Unable to get redirect")
                .into_response(StatusCode::INTERNAL_SERVER_ERROR)
                .map(|x| x.into_response())
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct UserDetails {
    pub username: String,
}

impl From<Option<&HeaderValue>> for UserDetails {
    fn from(value: Option<&HeaderValue>) -> Self {
        UserDetails {
            username: value
                .map(|x| x.to_str().unwrap())
                .map(|x| x.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

pub fn extract_user() -> impl Filter<Extract = (UserDetails,), Error = Infallible> + Clone {
    warp::filters::header::headers_cloned().map(|headers: HeaderMap| {
        trace!("Headers: {:?}", headers);
        if headers.contains_key("token-claim-sub") {
            UserDetails::from(headers.get("token-claim-sub"))
        } else if headers.contains_key("x-amzn-oidc-identity") {
            UserDetails::from(headers.get("x-amzn-oidc-identity"))
        } else {
            UserDetails {
                username: "unknown".to_string(),
            }
        }
    })
}
