use gadget_backend::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;
use warp::http::StatusCode;

// -- Errors

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    ClientError { client_error: ClientError },
    #[error(transparent)]
    ServerError { server_error: ServerError },
}

impl warp::reject::Reject for ApiError {}

pub trait StatusCodeError {
    fn status_code(&self) -> StatusCode;
}

impl From<BackendError> for ApiError {
    fn from(error: BackendError) -> Self {
        match error {
            BackendError::RedirectNotFound { search } => {
                ClientError::RedirectNotFound { search }.into()
            }
            BackendError::UserNotFound { id } => ClientError::UserNotFound { id }.into(),
            BackendError::RedirectAlreadyExists { alias } => {
                ClientError::RedirectAlreadyExists { alias }.into()
            }
            BackendError::InvalidNumber { message } => {
                ClientError::InvalidNumber { message }.into()
            }
            BackendError::SqlError { error } => ServerError::from_error("S-001", error).into(),
        }
    }
}

impl From<ServerError> for ApiError {
    fn from(error: ServerError) -> Self {
        ApiError::ServerError {
            server_error: error,
        }
    }
}

#[derive(Error, Debug)]
#[error("Internal Server Error {error_code}. Check logs for {reference_id}")]
pub struct ServerError {
    error_code: String,
    reference_id: String,
}

impl ServerError {
    pub fn from_error(error_code: &str, error: anyhow::Error) -> Self {
        let id = uuid::Uuid::new_v4();
        error!({error_id = %id.to_string(), error_code = %error_code}, "Server Error {}", error);

        Self {
            error_code: error_code.to_owned(),
            reference_id: id.to_string(),
        }
    }
}

impl StatusCodeError for ServerError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<ClientError> for ApiError {
    fn from(error: ClientError) -> Self {
        ApiError::ClientError {
            client_error: error,
        }
    }
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("The destination {destination} is not a valid URL. Destination must be a valid URL")]
    InvalidDestination { destination: String },
    #[error("Redirect not found matching `{search}`")]
    RedirectNotFound { search: String },
    #[error("User with id {id} not found")]
    UserNotFound { id: i32 },
    #[error("Redirect already exists for `{alias}`")]
    RedirectAlreadyExists { alias: String },
    #[error("Unable to convert number: {message}")]
    InvalidNumber { message: String },
    #[error("Endpoint required authentication, but none was provided.")]
    NoUserFound,
    #[error("Endpoint '{endpoint} was not found.")]
    ApiNotFound { endpoint: String },
}

impl StatusCodeError for ClientError {
    fn status_code(&self) -> StatusCode {
        match self {
            ClientError::UserNotFound { .. } | ClientError::RedirectNotFound { .. } | ClientError::ApiNotFound { .. }=> {
                StatusCode::NOT_FOUND
            }
            ClientError::InvalidDestination { .. } | ClientError::InvalidNumber { .. } => {
                StatusCode::BAD_REQUEST
            }
            ClientError::RedirectAlreadyExists { .. } => StatusCode::CONFLICT,
            ClientError::NoUserFound => StatusCode::FORBIDDEN,
        }
    }
}

// -- Shared API

#[derive(Debug, Serialize)]
pub struct PaginationState {
    #[serde(rename = "more")]
    pub has_more: bool,
    pub total: u32,
}

#[derive(Serialize)]
#[serde(remote = "StatusCode")]
struct StatusCodeDef {
    #[serde(getter = "StatusCode::as_u16")]
    pub code: u16,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum StatusResponse {
    Success(SuccessfulStatusResponse),
    Error(ErrorStatusResponse),
}

#[derive(Debug, Serialize)]
pub struct SuccessfulStatusResponse {
    #[serde(flatten, with = "StatusCodeDef")]
    pub code: StatusCode,
}

#[derive(Debug, Serialize)]
pub struct ErrorStatusResponse {
    #[serde(flatten, with = "StatusCodeDef")]
    pub code: StatusCode,
    pub error: Option<Vec<String>>,
}

impl ErrorStatusResponse {
    pub fn from_error_message(code: StatusCode, error: String) -> Self {
        Self {
            code,
            error: Some(vec![error]),
        }
    }
}

impl StatusResponse {
    pub fn ok() -> Self {
        StatusResponse::Success(SuccessfulStatusResponse {
            code: StatusCode::OK,
        })
    }

    pub fn status(&self) -> StatusCode {
        match self {
            StatusResponse::Error(err) => err.code,
            StatusResponse::Success(suc) => suc.code,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApplicationResponse<T>
where
    T: Serialize,
{
    pub status: StatusResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<PaginationState>,
}

#[test]
fn validate_create_repo_deserialize() {
    use json::object;

    let serialized = serde_json::to_string(&ApplicationResponse::<()> {
        status: StatusResponse::ok(),
        data: None,
        page: None,
    })
    .unwrap();

    assert_eq!(
        serialized,
        json::stringify(object! {
                "status": {
                    "code": 200
                },
        })
    );
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiPagination {
    #[serde(default = "default_0")]
    pub page: u32,
    #[serde(default = "default_50")]
    pub size: u32,
}

fn default_0() -> u32 {
    0
}

fn default_50() -> u32 {
    50
}

impl Default for ApiPagination {
    fn default() -> Self {
        Self { page: 0, size: 50 }
    }
}

#[derive(Deserialize, Serialize)]
pub struct DeleteStatus {
    pub deleted: bool,
}

impl From<bool> for DeleteStatus {
    fn from(source: bool) -> Self {
        Self { deleted: source }
    }
}

impl super::api::ApiResponse for DeleteStatus {}

pub struct PaginatedWrapperResponse<T>
where
    T: Serialize,
{
    pub data: T,
    pub page_options: Option<PaginationState>,
}

impl<T: Serialize> PaginatedWrapperResponse<T> {
    pub fn without_page(body: T) -> PaginatedWrapperResponse<T> {
        PaginatedWrapperResponse {
            data: body,
            page_options: None,
        }
    }

    pub fn with_page(body: T, total: u32, has_more: bool) -> PaginatedWrapperResponse<T> {
        PaginatedWrapperResponse {
            data: body,
            page_options: Some(PaginationState { total, has_more }),
        }
    }
}
