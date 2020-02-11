#[cfg(feature = "postgres")]
use super::schema::redirects;
use chrono::{NaiveDateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::iter;

#[cfg_attr(feature = "postgres", derive(Queryable))]
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
            public_ref: make_random_id(),
            alias: alias.to_string(),
            destination: destination.to_string(),
            created_on: Utc::now().naive_utc(),
            created_by,
        }
    }
}

#[cfg(feature = "postgres")]
#[derive(Insertable)]
#[table_name = "redirects"]
pub struct RedirectInsert<'a> {
    pub public_ref: String,
    pub alias: &'a str,
    pub destination: &'a str,
    pub created_on: NaiveDateTime,
    pub created_by: &'a str,
}

#[cfg(feature = "postgres")]
impl<'a> RedirectInsert<'a> {
    pub fn new(alias: &'a str, destination: &'a str, created_by: &'a str) -> Self {
        RedirectInsert {
            public_ref: make_random_id(),
            alias,
            destination,
            created_on: Utc::now().naive_utc(),
            created_by,
        }
    }
}

#[cfg(feature = "postgres")]
#[derive(AsChangeset)]
#[table_name = "redirects"]
pub struct RedirectUpdate<'a> {
    pub destination: &'a str,
    pub created_on: NaiveDateTime,
    pub created_by: &'a str,
}

#[cfg(feature = "postgres")]
impl<'a> RedirectUpdate<'a> {
    pub fn new(destination: &'a str, created_by: &'a str) -> Self {
        RedirectUpdate {
            destination,
            created_on: Utc::now().naive_utc(),
            created_by,
        }
    }
}

fn make_random_id() -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(10)
        .collect::<String>()
}
