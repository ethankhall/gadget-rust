use super::schema::redirects;
use chrono::{NaiveDateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::iter;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct RedirectModel {
    pub redirect_id: i32,
    pub public_ref: String,
    pub alias: String,
    pub destination: String,
    pub created_on: NaiveDateTime,
    pub created_by: Option<String>,
}

#[derive(Insertable)]
#[table_name = "redirects"]
pub struct RedirectInsert<'a> {
    pub public_ref: String,
    pub alias: &'a str,
    pub destination: &'a str,
    pub created_on: NaiveDateTime,
}

impl<'a> RedirectInsert<'a> {
    pub fn new(alias: &'a str, destination: &'a str) -> Self {
        RedirectInsert {
            public_ref: RedirectInsert::make_random_id(),
            alias,
            destination,
            created_on: Utc::now().naive_utc(),
        }
    }

    fn make_random_id() -> String {
        let mut rng = thread_rng();
        iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(10)
            .collect::<String>()
    }
}
