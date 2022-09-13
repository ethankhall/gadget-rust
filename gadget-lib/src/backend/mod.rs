mod json;
mod memory;
use crate::prelude::Result;

pub trait Backend<'a> {
    fn get_redirect(&self, redirect_ref: &str) -> Result<Option<RedirectModel>>;

    fn create_redirect(
        &self,
        new_alias: &str,
        new_destination: &str,
        username: &str,
    ) -> Result<RedirectModel>;

    fn update_redirect(&self, redirect_ref: &str, new_dest: &str, username: &str) -> Result<usize>;

    fn delete_redirect(&self, redirect_ref: &str) -> Result<usize>;

    fn get_all(&self, page: u64, limit: usize) -> Result<Vec<RedirectModel>>;
}

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct RedirectModel {
    pub redirect_id: i32,
    pub public_ref: String,
    pub alias: String,
    pub destination: String,
    pub created_on: NaiveDateTime,
    pub created_by: Option<String>,
}

impl RedirectModel {
    pub fn set_destination(&mut self, destination: &str) {
        self.destination = destination.to_string();
    }

    pub fn update_username(&mut self, username: Option<&str>) {
        self.created_by = username.map(|x| x.to_string());
    }

    pub fn new(id: i32, alias: &str, destination: &str, created_by: Option<String>) -> Self {
        RedirectModel {
            redirect_id: id,
            public_ref: "".to_string(),
            alias: alias.to_string(),
            destination: destination.to_string(),
            created_on: Utc::now().naive_utc(),
            created_by,
        }
    }
}

pub mod prelude {
    pub use super::json::JsonBackend;
    pub use super::memory::InMemoryBackend;
    pub use super::{Backend, RedirectModel};
}
