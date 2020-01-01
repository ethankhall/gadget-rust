use std::sync::Arc;

use actix_web::{http::{StatusCode, header::LOCATION}, web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::backend::{Backend, BackendContainer, RowChange, RedirectModel};
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

#[derive(Serialize)]
pub struct ApiRedirect {
    id: String,
    alias: String,
    destination: String
}

impl Into<ApiRedirect> for RedirectModel {
    fn into(self) -> ApiRedirect {
        ApiRedirect {
            id: self.public_ref,
            alias: self.alias,
            destination: self.destination
        }
    }
}

#[derive(Serialize)]
struct RedirectList {
    redirects: Vec<ApiRedirect>
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
        RowChange::Value(result) => {
            let api_model: ApiRedirect = result.into();
            HttpResponse::build(StatusCode::CREATED).json(api_model)
        },
        RowChange::Err(e) => {
            warn!("Unable to create redirect: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Unable to create redirect: {:?}", e))
        }
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

pub async fn list_redirects(context: web::Data<Context>) -> HttpResponse {
    let resp = match context.backend.get_all(0, 10000) {
        RowChange::Value(v) => {
            let data: Vec<ApiRedirect> = v.into_iter().map(|x| x.into()).collect();
            RedirectList { redirects: data }
        },
        RowChange::NotFound => RedirectList { redirects: vec!() },
        RowChange::Err(e) => {
            warn!("Unable to get redirect: {:?}", e);
            return HttpResponse::InternalServerError().body("Unable to get redirect");
        }
    };

    HttpResponse::build(StatusCode::OK).json(resp)
}

pub async fn get_redirect(info: web::Path<String>, context: web::Data<Context>) -> HttpResponse {
    match context.backend.get_redirect(&info) {
        RowChange::Value(value) => {
            let redirect: ApiRedirect = value.into();
            HttpResponse::Ok().json(redirect)
        }
        RowChange::NotFound => HttpResponse::NotFound().finish(),
        RowChange::Err(e) => {
            warn!("Unable to get redirect: {:?}", e);
            HttpResponse::InternalServerError().body("Unable to get redirect")
        }
    }
}

pub async fn find_redirect(info: web::Path<String>, context: web::Data<Context>) -> HttpResponse {
    match context.backend.get_redirect(&info) {
        RowChange::Value(value) => {
            let redirect = AliasRedirect::from(value);
            HttpResponse::TemporaryRedirect()
                .header(LOCATION, redirect.get_destination(&info))
                .finish()
        }
        RowChange::NotFound => HttpResponse::TemporaryRedirect()
            .header(LOCATION, format!("/_gadget/ui?search={}", &info))
            .finish(),
        RowChange::Err(e) => {
            warn!("Unable to get redirect: {:?}", e);
            HttpResponse::InternalServerError().body("Unable to get redirect")
        }
    }
}
