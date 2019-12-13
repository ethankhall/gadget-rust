mod postgres;
mod models;
mod schema;
mod json;

pub use models::RedirectModel;

pub enum BackendContainer {
    Postgres(postgres::PostgresBackend),
}

impl BackendContainer {
    pub fn new<T: ToString>(url: T) -> Self {
        BackendContainer::Postgres(postgres::PostgresBackend::new(url))
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
    ) -> RowChange<RedirectModel>;

    fn update_redirect(&self, redirect_ref: &str, new_dest: &str) -> RowChange<usize>;

    fn delete_redirect(&self, redirect_ref: &str) -> RowChange<usize>;

    fn get_all(&self, page: i64, limit: i64) -> Result<Vec<RedirectModel>, String>;
}

impl Backend for BackendContainer {
    fn get_redirect(&self, redirect_ref: &str) -> RowChange<RedirectModel> {
        match self {
            BackendContainer::Postgres(p) => p.get_redirect(redirect_ref),
        }
    }

    fn create_redirect(
        &self,
        new_alias: &str,
        new_destination: &str,
    ) -> RowChange<RedirectModel> {
        match self {
            BackendContainer::Postgres(p) => p.create_redirect(new_alias, new_destination),
        }
    }

    fn delete_redirect(&self, redirect_ref: &str) -> RowChange<usize> {
        match self {
            BackendContainer::Postgres(p) => p.delete_redirect(redirect_ref),
        }
    }

    fn update_redirect(&self, redirect_ref: &str, new_dest: &str) -> RowChange<usize> {
        match self {
            BackendContainer::Postgres(p) => p.update_redirect(redirect_ref, new_dest),
        }
    }

    fn get_all(&self, page: i64, limit: i64) -> Result<Vec<RedirectModel>, String> {
        match self {
            BackendContainer::Postgres(p) => p.get_all(page, limit),
        }
    }
}
