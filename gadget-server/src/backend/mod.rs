use std::ops::Deref;

mod json;
mod models;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
mod schema;

pub use models::RedirectModel;

pub enum BackendContainer {
    Json(json::JsonBackend),
    #[cfg(feature = "postgres")]
    Postgres(postgres::PostgresBackend),
}

impl Deref for BackendContainer {
    type Target = dyn Backend;

    fn deref(&self) -> &Self::Target {
        match self {
            BackendContainer::Json(j) => j,
            #[cfg(feature = "postgres")]
            BackendContainer::Postgres(p) => p,
        }
    }
}

impl BackendContainer {
    #[cfg(not(feature = "postgres"))]
    pub fn new(url: String) -> Result<Self, String> {
        if url.starts_with("file://") {
            Ok(BackendContainer::Json(json::JsonBackend::new(url)))
        } else {
            Err("Database path must start file://".to_string())
        }
    }

    #[cfg(feature = "postgres")]
    pub fn new(url: String) -> Result<Self, String> {
        if url.starts_with("postgresql://") {
            Ok(BackendContainer::Postgres(postgres::PostgresBackend::new(
                url,
            )))
        } else if url.starts_with("file://") {
            Ok(BackendContainer::Json(json::JsonBackend::new(url)))
        } else {
            Err("Database path must start file:// or postgresql://".to_string())
        }
    }
}

pub enum RowChange<T> {
    NotFound,
    Value(T),
    Err(String),
}

pub trait Backend {
    fn get_redirect(&self, redirect_ref: &str) -> RowChange<RedirectModel>;

    fn create_redirect(
        &self,
        new_alias: &str,
        new_destination: &str,
        username: &str,
    ) -> RowChange<RedirectModel>;

    fn update_redirect(
        &self,
        redirect_ref: &str,
        new_dest: &str,
        username: &str,
    ) -> RowChange<usize>;

    fn delete_redirect(&self, redirect_ref: &str) -> RowChange<usize>;

    fn get_all(&self, page: u64, limit: usize) -> RowChange<Vec<RedirectModel>>;
}
