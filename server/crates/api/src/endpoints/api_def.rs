use crate::{endpoints::api_actions::*, extractors::extract_user, models::*, SharedContext};
use serde::de::DeserializeOwned;
use warp::{Filter, Rejection, Reply};

pub fn api_endpoint(
    backend: SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    api_favicon()
        .or(api_list_redirects(&backend))
        .or(api_new_redirect_json(&backend))
        .or(api_delete_redirect(&backend))
        .or(api_update_redirect(&backend))
        .or(api_get_redirect(&backend))
        .or(api_not_found())
        .or(api_find_redirect(&backend))
        .with(warp::log::custom(super::metrics::track_status))
}

fn api_favicon() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("favicon.ico").and_then(favicon)
}

fn api_not_found() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("_gadget")
    .and(warp::path::tail())
    .and_then(|tail: warp::filters::path::Tail| {
        standardize_output::<ApiRedirect, ClientError>(Err(ClientError::ApiNotFound {
            endpoint: tail.as_str().to_owned(),
        }))
    })
}

fn api_list_redirects(
    backend: &SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("_gadget" / "api" / "redirect")
        .and(warp::get())
        .and(warp::query::<ApiPagination>())
        .and(extract_user())
        .and(with_context(backend.clone()))
        .and_then(|pagination, user, context| async {
            standardize_output(list_redirects(pagination, user, context).await).await
        })
}

fn api_new_redirect_json(
    backend: &SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("_gadget" / "api" / "redirect")
        .and(warp::post())
        .and(json_body())
        .and(extract_user())
        .and(with_context(backend.clone()))
        .and_then(|redirect, user, context| async {
            standardize_output(new_redirect_json(redirect, user, context).await).await
        })
}

fn api_delete_redirect(
    backend: &SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("_gadget" / "api" / "redirect" / String)
        .and(warp::delete())
        .and(extract_user())
        .and(with_context(backend.clone()))
        .and_then(|path, user, context| async {
            standardize_output(delete_redirect(path, user, context).await).await
        })
}

fn api_update_redirect(
    backend: &SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("_gadget" / "api" / "redirect" / String)
        .and(warp::put())
        .and(json_body())
        .and(extract_user())
        .and(with_context(backend.clone()))
        .and_then(|path, destination, user, context| async {
            standardize_output(update_redirect(path, destination, user, context).await).await
        })
}

fn api_get_redirect(
    backend: &SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("_gadget" / "api" / "redirect" / String)
        .and(warp::get())
        .and(extract_user())
        .and(with_context(backend.clone()))
        .and_then(|path, user, context| async {
            standardize_output(get_redirect(path, user, context).await).await
        })
}

fn api_find_redirect(
    backend: &SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(warp::path::tail())
        .and(with_context(backend.clone()))
        .and_then(find_redirect)
}

fn with_context(
    context: SharedContext,
) -> impl Filter<Extract = (SharedContext,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || context.clone())
}

fn json_body<T: DeserializeOwned + Send>(
) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

async fn standardize_output<O, E>(
    res: Result<PaginatedWrapperResponse<O>, E>,
) -> Result<impl Reply, Rejection>
where
    O: ApiResponse,
    E: Into<ApiError>,
{
    match res {
        Ok(body) => {
            let response = ApplicationResponse {
                status: StatusResponse::ok(),
                data: Some(body.data),
                page: body.page_options,
            };

            Ok(warp::reply::json(&response))
        }
        Err(error) => {
            let api_error: ApiError = error.into();
            Err(warp::reject::custom(api_error))
        }
    }
}
