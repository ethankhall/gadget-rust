use chrono::{DateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::iter;

#[derive(Debug)]
pub struct DbRedirect {
    pub public_ref: String,
    pub alias: String,
    pub destination: String,
    pub created_on: DateTime<Utc>,
}

impl From<&crate::entity::global_redirects::Model> for DbRedirect {
    fn from(model: &crate::entity::global_redirects::Model) -> Self {
        Self {
            public_ref: model.public_ref.to_owned(),
            alias: model.alias.to_owned(),
            destination: model.destination.to_owned(),
            created_on: DateTime::<Utc>::from_utc(model.created_on, Utc),
        }
    }
}

#[derive(Debug)]
pub struct DbNewRedirect {
    pub(crate) public_ref: String,
    pub(crate) alias: String,
    pub(crate) destination: String,
    pub(crate) created_on: DateTime<Utc>,
}

impl DbNewRedirect {
    pub fn new(alias: &str, destination: &str) -> Self {
        Self {
            public_ref: make_random_id(),
            alias: alias.to_string(),
            destination: destination.to_string(),
            created_on: Utc::now(),
        }
    }
}

#[derive(Debug)]
pub struct DbUpdateRedirect {
    pub(crate) public_ref: String,
    pub(crate) destination: String,
    pub(crate) created_on: DateTime<Utc>,
}

impl DbUpdateRedirect {
    pub fn new(public_ref: &str, destination: &str) -> Self {
        Self {
            public_ref: public_ref.to_owned(),
            destination: destination.to_string(),
            created_on: Utc::now(),
        }
    }
}

#[derive(Debug)]
pub struct DbUser {
    pub external_id: String,
    pub name: String,
}

impl DbUser {
    pub fn new(external_id: &str, name: &str) -> Self {
        Self {
            external_id: external_id.to_owned(),
            name: name.to_owned(),
        }
    }
}

impl From<&crate::entity::external_user::Model> for DbUser {
    fn from(user: &crate::entity::external_user::Model) -> Self {
        Self {
            external_id: user.external_user_id.to_owned(),
            name: user.prefered_name.to_owned(),
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

#[derive(Debug, Clone)]
pub struct PaginationOptions {
    pub page_number: u32,
    pub page_size: u32,
}

impl PaginationOptions {
    pub fn new(page_number: u32, page_size: u32) -> Self {
        Self {
            page_number,
            page_size,
        }
    }
}

#[derive(Debug)]
pub struct DbRedirectUserPair {
    pub redirect: DbRedirect,
    pub user: DbUser,
}

impl DbRedirectUserPair {
    pub fn new(redirect: DbRedirect, user: DbUser) -> Self {
        Self { redirect, user }
    }
}
