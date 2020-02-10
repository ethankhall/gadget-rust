mod json;
mod models;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
mod schema;

pub use models::RedirectModel;

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

#[cfg(not(feature = "postgres"))]
pub fn make_backend(url: String) -> Result<impl Backend, String> {
    if url.starts_with("file://") {
        Ok(json::JsonBackend::new(url))
    } else {
        Err("Database path must start file://".to_string())
    }
}

#[cfg(feature = "postgres")]
pub fn make_backend(url: String) -> impl Backend {
    if url.starts_with("file://") {
        postgres::PostgresBackend::new(url)
    } else {
        error!("Database path must start file://");
        std::process::exit(1);
    }
}
