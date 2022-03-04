use tracing::{debug, info, instrument};

use url::Url;

use crate::redirect::{AliasRedirect, Redirect};
use crate::{
    models::{
        ApiError, ApiNewRedirect, ApiPagination, ApiRedirect, ApiRedirectList, ApiResponse,
        ApiUpdateRedirect, ApiUser, ClientError, DeleteStatus, PaginatedWrapperResponse,
    },
    SharedContext,
};
use gadget_backend::prelude::BackendError;
use std::convert::Infallible;
use warp::{http::StatusCode, Rejection};

pub async fn favicon() -> Result<impl warp::Reply, Infallible> {
    Ok(StatusCode::NOT_FOUND)
}

#[instrument(skip(context))]
pub async fn delete_redirect(
    path: String,
    user: Option<ApiUser>,
    context: SharedContext,
) -> Result<PaginatedWrapperResponse<impl ApiResponse>, ApiError> {
    if user.is_none() {
        return Err(ApiError::from(ClientError::NoUserFound));
    }

    context
        .backend
        .delete_redirect(&path)
        .await
        .map(|_| DeleteStatus::from(true))
        .map(PaginatedWrapperResponse::without_page)
        .map_err(ApiError::from)
}

#[instrument(skip(context))]
pub async fn new_redirect_json(
    info: ApiNewRedirect,
    user: Option<ApiUser>,
    context: SharedContext,
) -> Result<PaginatedWrapperResponse<impl ApiResponse>, ApiError> {
    if !is_destination_url(&info.destination) {
        debug!("Destination wasn't URL {:?}", &info.destination);
        return Err(ApiError::from(ClientError::InvalidDestination {
            destination: info.destination,
        }));
    }

    let user = match user {
        Some(user) => user,
        None => return Err(ApiError::from(ClientError::NoUserFound)),
    };

    info!("Creating redirect {} => {}", info.alias, info.destination);
    context
        .backend
        .create_redirect(&info.alias, &info.destination, &user.into())
        .await
        .map(|redirect| ApiRedirect::from(&redirect))
        .map(PaginatedWrapperResponse::without_page)
        .map_err(ApiError::from)
}

#[instrument(skip(context))]
pub async fn update_redirect(
    alias: String,
    dest: ApiUpdateRedirect,
    user: Option<ApiUser>,
    context: SharedContext,
) -> Result<PaginatedWrapperResponse<impl ApiResponse>, ApiError> {
    if !is_destination_url(&dest.destination) {
        debug!("Destination wasn't URL {:?}", &dest.destination);
        return Err(ApiError::from(ClientError::InvalidDestination {
            destination: dest.destination,
        }));
    }

    let user = match user {
        Some(user) => user,
        None => return Err(ApiError::from(ClientError::NoUserFound)),
    };

    context
        .backend
        .update_redirect(&alias, &dest.destination, &user.into())
        .await
        .map(|redirect| ApiRedirect::from(&redirect))
        .map(PaginatedWrapperResponse::without_page)
        .map_err(ApiError::from)
}

#[instrument(skip(context))]
pub async fn list_redirects(
    pagination: ApiPagination,
    user: Option<ApiUser>,
    context: SharedContext,
) -> Result<PaginatedWrapperResponse<impl ApiResponse>, ApiError> {
    if user.is_none() {
        return Err(ApiError::from(ClientError::NoUserFound));
    }

    let result = context.backend.get_all(pagination.into()).await;
    result
        .map(|list| {
            (
                ApiRedirectList {
                    redirects: list.redirects.iter().map(ApiRedirect::from).collect(),
                },
                list.total_count,
                list.has_more,
            )
        })
        .map(|(body, total_count, has_more)| {
            PaginatedWrapperResponse::with_page(body, total_count, has_more)
        })
        .map_err(ApiError::from)
}

fn is_destination_url(path: &str) -> bool {
    Url::parse(path).is_ok()
}

#[instrument(skip(context))]
pub async fn get_redirect(
    alias: String,
    user: Option<ApiUser>,
    context: SharedContext,
) -> Result<PaginatedWrapperResponse<impl ApiResponse>, ApiError> {
    if user.is_none() {
        return Err(ApiError::from(ClientError::NoUserFound));
    }

    context
        .backend
        .get_redirect(&alias)
        .await
        .map(|redirect| ApiRedirect::from(&redirect))
        .map(PaginatedWrapperResponse::without_page)
        .map_err(ApiError::from)
}

#[tracing::instrument(skip(context))]
pub async fn find_redirect(
    path: warp::filters::path::Tail,
    context: SharedContext,
) -> Result<warp::reply::Response, Rejection> {
    use warp::{
        http::{header::LOCATION, StatusCode},
        hyper::Body,
    };

    let path = path.as_str().replace("%20", " ");

    if path.starts_with("_gadget") {
        info!("Found _gadget path `{}`, defaulting to 404", path);
        return Err(warp::reject::custom(ApiError::from(ClientError::ApiNotFound {
            endpoint: path.to_owned()
        })));
    }

    let redirect_ref: Vec<&str> = path.split(' ').collect();
    let redirect_ref = match redirect_ref.first() {
        None => {
            return Ok(warp::http::Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, context.ui_location.to_string())
                .body(Body::empty())
                .unwrap());
        }
        Some(value) => value,
    };

    match context.backend.get_redirect(redirect_ref).await {
        Ok(value) => {
            let redirect = AliasRedirect::from(value);
            Ok(warp::http::Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, redirect.get_destination(&path))
                .body(Body::empty())
                .unwrap())
        }
        Err(BackendError::RedirectNotFound { .. }) => {
            let mut target_url = context.ui_location.clone();
            target_url.query_pairs_mut().append_pair("search", &path);
            Ok(warp::http::Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, target_url.to_string())
                .body(Body::empty())
                .unwrap())
        }
        Err(err) => Err(warp::reject::custom(ApiError::from(err))),
    }
}
