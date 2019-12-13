use std::convert::TryFrom;
use std::sync::Arc;

use actix_web::{http::header::LOCATION, web, HttpResponse};
use serde::Deserialize;

use crate::backend::{Backend, BackendContainer, RowChange};
use crate::redirect::{AliasRedirect, Redirect};

#[derive(Clone)]
pub struct Context {
    backend: Arc<BackendContainer>,
}

impl Context {
    pub fn new<T: ToString>(url: T) -> Self {
        Context {
            backend: Arc::new(BackendContainer::new(url)),
        }
    }
}

#[derive(Deserialize)]
pub struct NewRedirect {
    alias: String,
    destination: String,
}

#[derive(Deserialize)]
pub struct UpdateRedirect {
    destination: String,
}

impl Into<HttpResponse> for RowChange<usize> {
    fn into(self) -> HttpResponse {
        match self {
            RowChange::NotFound => HttpResponse::NotFound().body("Alais not found"),
            RowChange::Value(_) => HttpResponse::Ok().finish(),
            RowChange::Err(e) => HttpResponse::InternalServerError().body(e),
        }
    }
}

pub async fn favicon() -> HttpResponse {
    HttpResponse::NotFound().finish()
}

pub async fn delete_redirect(path: web::Path<String>, context: web::Data<Context>) -> HttpResponse {
    context.backend.delete_redirect(&path).into()
}

pub async fn new_redirect_json(
    info: web::Json<NewRedirect>,
    context: web::Data<Context>,
) -> HttpResponse {
    info!("Creating redirect {} => {}", info.alias, info.destination);
    match context
        .backend
        .create_redirect(&info.alias, &info.destination)
    {
        RowChange::Value(result) => HttpResponse::SeeOther()
            .header(LOCATION, format!("/_gadget/ui?redirect={}", result.alias))
            .finish(),
        RowChange::Err(e) => {
            warn!("Unable to create redirect: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Unable to create redirect: {:?}", e))
        },
        RowChange::NotFound => {
            warn!("Unable to create redirect");
            HttpResponse::InternalServerError().body("Unable to create redirect: {:?}")
        }
    }
}

pub async fn update_redirect(
    info: web::Path<String>,
    dest: web::Json<UpdateRedirect>,
    context: web::Data<Context>,
) -> HttpResponse {
    context
        .backend
        .update_redirect(&info, &dest.destination)
        .into()
}

pub async fn find_redirect(info: web::Path<String>, context: web::Data<Context>) -> HttpResponse {
    info!("Path: {}", &info);
    match context.backend.get_redirect(&info) {
        RowChange::Value(value) => match AliasRedirect::try_from(value) {
            Ok(redirect) => HttpResponse::TemporaryRedirect()
                .header(LOCATION, redirect.get_destination(&info))
                .finish(),
            Err(e) => {
                warn!("Unable to get redirect: {:?}", e);
                HttpResponse::InternalServerError().body("Unable to get redirect")
            }
        },
        RowChange::NotFound => HttpResponse::NotFound().finish(),
        RowChange::Err(e) => {
            warn!("Unable to get redirect: {:?}", e);
            HttpResponse::InternalServerError().body("Unable to get redirect")
        }
    }
}
