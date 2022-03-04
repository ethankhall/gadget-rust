use chrono::{DateTime, Utc};
use gadget_backend::prelude::*;
use serde::{Deserialize, Serialize};

pub trait ApiResponse: Serialize {}

#[derive(Deserialize, Debug)]
pub struct ApiNewRedirect {
    pub alias: String,
    pub destination: String,
}

#[derive(Deserialize, Debug)]
pub struct ApiUpdateRedirect {
    pub destination: String,
}

#[derive(Serialize)]
pub struct ApiRedirect {
    pub public_ref: String,
    pub alias: String,
    pub destination: String,
    pub created_on: DateTime<Utc>,
    pub created_by: ApiUser,
}

impl From<super::server::ApiPagination> for BackendPaginationOptions {
    fn from(pagination: super::server::ApiPagination) -> Self {
        Self {
            page_number: pagination.page,
            page_size: pagination.size,
        }
    }
}

impl ApiResponse for ApiRedirect {}

impl From<&BackendRedirect> for ApiRedirect {
    fn from(model: &BackendRedirect) -> Self {
        Self {
            public_ref: model.public_ref.to_owned(),
            alias: model.alias.to_owned(),
            destination: model.destination.to_owned(),
            created_on: model.created_on.to_owned(),
            created_by: model.created_by.clone().into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiUser {
    pub name: String,
    pub id: String,
}

impl From<BackendRedirectUser> for ApiUser {
    fn from(user: BackendRedirectUser) -> Self {
        Self {
            name: user.name,
            id: user.external_id,
        }
    }
}

impl From<ApiUser> for BackendRedirectUser {
    fn from(user: ApiUser) -> Self {
        Self {
            name: user.name,
            external_id: user.id,
        }
    }
}

#[derive(Serialize)]
pub struct ApiRedirectList {
    pub redirects: Vec<ApiRedirect>,
}

impl ApiResponse for ApiRedirectList {}
