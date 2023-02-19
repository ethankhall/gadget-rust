mod json;
mod memory;
use crate::prelude::LibResult;

pub trait Backend<'a> {
    fn get_redirect(&self, redirect_ref: &str) -> LibResult<Option<RedirectModel>>;

    fn create_redirect(
        &self,
        new_alias: &str,
        new_destination: &str,
        username: &str,
    ) -> LibResult<RedirectModel>;

    fn update_redirect(
        &self,
        redirect_ref: &str,
        new_dest: &str,
        username: &str,
    ) -> LibResult<RedirectModel>;

    fn delete_redirect(&self, redirect_ref: &str) -> LibResult<usize>;

    fn get_all(&self, page: u64, limit: usize) -> LibResult<Vec<RedirectModel>>;
}

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
            public_ref: make_random_id(),
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

#[cfg(not(target_arch = "wasm32"))]
fn make_random_id() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;

    let mut rng = thread_rng();
    let bytes: Vec<u8> = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(10)
        .collect();

    String::from_utf8(bytes).expect("Found invalid UTF-8")
}

#[cfg(target_arch = "wasm32")]
fn make_random_id() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{rngs::OsRng, Rng};
    use std::iter;

    let bytes: Vec<u8> = iter::repeat(())
        .map(|()| OsRng.sample(Alphanumeric))
        .take(10)
        .collect();

    String::from_utf8(bytes).expect("Found invalid UTF-8")
}
