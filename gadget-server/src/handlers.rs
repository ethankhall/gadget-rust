use gadget_lib::api::*;
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, trace, warn};

use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use warp::{
    http::header::LOCATION,
    http::{HeaderMap, HeaderValue, StatusCode},
    reply::Reply,
    Filter,
};

use gadget_lib::prelude::{AliasRedirect, Backend, GadgetLibError, Redirect};

#[derive(Clone)]
pub struct RequestContext<'a> {
    backend: Arc<Box<dyn Backend<'a>>>,
}

unsafe impl std::marker::Send for RequestContext<'_> {}

unsafe impl std::marker::Sync for RequestContext<'_> {}

impl<'a> RequestContext<'a> {
    pub fn new(backend: Box<dyn Backend<'a>>) -> Self {
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

pub async fn favicon() -> Result<impl warp::Reply, Infallible> {
    Ok(StatusCode::NOT_FOUND)
}

#[instrument(skip(context))]
pub async fn delete_redirect(
    path: String,
    context: Arc<RequestContext<'_>>,
) -> Result<impl warp::Reply, Infallible> {
    let resp = context.backend.delete_redirect(&path);

    match resp {
        Err(GadgetLibError::RedirectDoesNotExists(_)) => {
            ResponseMessage::from("not found").into_response(StatusCode::NOT_FOUND)
        }
        Ok(_) => ResponseMessage::from("ok").into_response(StatusCode::OK),
        Err(e) => {
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
    info: ApiRedirect,
    user: UserDetails,
    context: Arc<RequestContext<'_>>,
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
        Ok(result) => {
            let api_model: ApiRedirect = result.into();
            Ok(warp::reply::with_status(
                warp::reply::json(&api_model),
                StatusCode::CREATED,
            ))
        }
        Err(e) => {
            warn!("Unable to create redirect: {:?}", e);
            ResponseMessage::from(format!("Unable to create redirect: {:?}", e))
                .into_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[instrument(skip(context))]
pub async fn update_redirect(
    info: String,
    dest: UpdateRedirect,
    user: UserDetails,
    context: Arc<RequestContext<'_>>,
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
        Ok(_) => ResponseMessage::from("ok").into_response(StatusCode::OK),
        Err(GadgetLibError::RedirectDoesNotExists(_)) => {
            ResponseMessage::from("not found").into_response(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Unable to update redirect: {:?}", e);
            ResponseMessage::from(format!("Unexpected error: {:?}", e))
                .into_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[instrument(skip(context))]
pub async fn list_redirects(
    context: Arc<RequestContext<'_>>,
) -> Result<impl warp::Reply, Infallible> {
    let resp = match context.backend.get_all(0, 10000) {
        Ok(v) => {
            let data: Vec<ApiRedirect> = v.into_iter().map(|x| x.into()).collect();
            RedirectList { redirects: data }
        }
        Err(GadgetLibError::RedirectDoesNotExists(_)) => RedirectList { redirects: vec![] },
        Err(e) => {
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
    Url::parse(path).is_ok()
}

#[instrument(skip(context))]
pub async fn get_redirect(
    info: String,
    context: Arc<RequestContext<'_>>,
) -> Result<impl warp::Reply, Infallible> {
    match context.backend.get_redirect(&info) {
        Ok(Some(value)) => {
            let redirect: ApiRedirect = value.into();
            Ok(warp::reply::with_status(
                warp::reply::json(&redirect),
                StatusCode::OK,
            ))
        }
        Ok(None) | Err(GadgetLibError::RedirectDoesNotExists(_)) => {
            ResponseMessage::from("Unable to get redirect").into_response(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            warn!("Unable to get redirect: {:?}", e);
            ResponseMessage::from("Unable to get redirect")
                .into_response(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tracing::instrument(skip(context))]
pub async fn find_redirect(
    path: warp::filters::path::Tail,
    context: Arc<RequestContext<'_>>,
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
        Ok(Some(value)) => {
            let redirect = AliasRedirect::from(value);
            Ok(warp::http::Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, redirect.get_destination(&info))
                .body(hyper::Body::empty())
                .unwrap())
        }
        Ok(None) | Err(GadgetLibError::RedirectDoesNotExists(_)) => {
            Ok(warp::http::Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, format!("/_gadget/ui?search={}", &info))
                .body(hyper::Body::empty())
                .unwrap())
        }
        Err(e) => {
            warn!("Unable to get redirect: {:?}", e);
            ResponseMessage::from("Unable to get redirect")
                .into_response(StatusCode::INTERNAL_SERVER_ERROR)
                .map(|x| x.into_response())
        }
    }
}

fn extract_user_details(value: Option<&'_ HeaderValue>) -> UserDetails {
    UserDetails {
        username: value
            .map(|x| x.to_str().unwrap())
            .map(|x| x.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
    }
}

pub fn extract_user() -> impl Filter<Extract = (UserDetails,), Error = Infallible> + Clone {
    warp::filters::header::headers_cloned().map(|headers: HeaderMap| {
        trace!("Headers: {:?}", headers);
        if headers.contains_key("token-claim-sub") {
            extract_user_details(headers.get("token-claim-sub"))
        } else if headers.contains_key("x-amzn-oidc-identity") {
            extract_user_details(headers.get("x-amzn-oidc-identity"))
        } else {
            UserDetails {
                username: "unknown".to_string(),
            }
        }
    })
}
