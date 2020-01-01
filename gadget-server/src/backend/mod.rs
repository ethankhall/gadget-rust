mod json;
mod models;
mod postgres;
mod schema;

pub use models::RedirectModel;

pub enum BackendContainer {
    Postgres(postgres::PostgresBackend),
    Json(json::JsonBackend),
}

impl BackendContainer {
    pub fn new<T: ToString>(url: T) -> Self {
        let url = url.to_string();
        if url.starts_with("postgresql://") {
            return BackendContainer::Postgres(postgres::PostgresBackend::new(url));
        } else if url.starts_with("file://") {
            return BackendContainer::Json(json::JsonBackend::new(url));
        } else {
            error!("Database path must start with either postgresql:// or file://");
            panic!();
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

    fn create_redirect(&self, new_alias: &str, new_destination: &str) -> RowChange<RedirectModel>;

    fn update_redirect(&self, redirect_ref: &str, new_dest: &str) -> RowChange<usize>;

    fn delete_redirect(&self, redirect_ref: &str) -> RowChange<usize>;

    fn get_all(&self, page: u64, limit: usize) -> RowChange<Vec<RedirectModel>>;
}

impl Backend for BackendContainer {
    fn get_redirect(&self, redirect_ref: &str) -> RowChange<RedirectModel> {
        match self {
            BackendContainer::Postgres(p) => p.get_redirect(redirect_ref),
            BackendContainer::Json(j) => j.get_redirect(redirect_ref),
        }
    }

    fn create_redirect(&self, new_alias: &str, new_destination: &str) -> RowChange<RedirectModel> {
        match self {
            BackendContainer::Postgres(p) => p.create_redirect(new_alias, new_destination),
            BackendContainer::Json(j) => j.create_redirect(new_alias, new_destination),
        }
    }

    fn delete_redirect(&self, redirect_ref: &str) -> RowChange<usize> {
        match self {
            BackendContainer::Postgres(p) => p.delete_redirect(redirect_ref),
            BackendContainer::Json(j) => j.delete_redirect(redirect_ref),
        }
    }

    fn update_redirect(&self, redirect_ref: &str, new_dest: &str) -> RowChange<usize> {
        match self {
            BackendContainer::Postgres(p) => p.update_redirect(redirect_ref, new_dest),
            BackendContainer::Json(j) => j.update_redirect(redirect_ref, new_dest),
        }
    }

    fn get_all(&self, page: u64, limit: usize) -> RowChange<Vec<RedirectModel>> {
        match self {
            BackendContainer::Postgres(p) => p.get_all(page, limit),
            BackendContainer::Json(j) => j.get_all(page, limit),
        }
    }
}
