use crate::models::*;
use std::convert::Infallible;
use std::error::Error;
use tracing::error;
use warp::{http::StatusCode, reject::Reject, Rejection, Reply};

impl From<Rejection> for ErrorStatusResponse {
    fn from(source: Rejection) -> Self {
        if source.is_not_found() {
            ErrorStatusResponse::from_error_message(StatusCode::NOT_FOUND, "NOT_FOUND".to_owned())
        } else if let Some(resp) = source.find::<ErrorStatusResponse>() {
            resp.into()
        } else if let Some(api_error) = source.find::<ApiError>() {
            api_error.into()
        } else if let Some(e) = source.find::<warp::filters::body::BodyDeserializeError>() {
            let message_body: String = match e.source() {
                Some(cause) => cause.to_string(),
                None => "BAD_REQUEST".into(),
            };
            ErrorStatusResponse::from_error_message(StatusCode::BAD_REQUEST, message_body)
        } else if source.find::<warp::reject::MethodNotAllowed>().is_some() {
            ErrorStatusResponse::from_error_message(
                StatusCode::METHOD_NOT_ALLOWED,
                "METHOD_NOT_ALLOWED".to_owned(),
            )
        } else {
            error!("unhandled rejection: {:?}", source);
            ErrorStatusResponse::from_error_message(
                StatusCode::INTERNAL_SERVER_ERROR,
                "UNHANDLED_REJECTION".to_owned(),
            )
        }
    }
}

impl From<&ErrorStatusResponse> for ErrorStatusResponse {
    fn from(source: &ErrorStatusResponse) -> Self {
        Self {
            code: source.code,
            error: source.error.clone(),
        }
    }
}

impl From<&ApiError> for ErrorStatusResponse {
    fn from(api_error: &ApiError) -> Self {
        let code = match api_error {
            ApiError::ClientError { client_error } => client_error.status_code(),
            ApiError::ServerError { server_error } => server_error.status_code(),
        };

        Self {
            code,
            error: Some(vec![api_error.to_string()]),
        }
    }
}

impl Reject for ErrorStatusResponse {}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let status = StatusResponse::Error(err.into());
    let status_code = status.status();
    let response: ApplicationResponse<()> = ApplicationResponse {
        data: None,
        status,
        page: None,
    };

    let json = warp::reply::json(&response);

    Ok(warp::reply::with_status(json, status_code))
}
