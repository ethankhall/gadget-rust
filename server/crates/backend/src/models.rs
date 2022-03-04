use chrono::{DateTime, Utc};
use gadget_database::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::iter;

#[derive(Debug, Clone)]
pub struct BackendPaginationOptions {
    pub page_number: u32,
    pub page_size: u32,
}

impl BackendPaginationOptions {
    pub fn new(page_number: u32, page_size: u32) -> Self {
        Self {
            page_number,
            page_size,
        }
    }

    pub fn has_more(&self, total: u32) -> bool {
        (1 + self.page_number) * self.page_size < total
    }
}

#[test]
fn validate_has_more() {
    assert_eq!(BackendPaginationOptions::new(0, 50).has_more(100), true);
    assert_eq!(BackendPaginationOptions::new(0, 50).has_more(10), false);
    assert_eq!(BackendPaginationOptions::new(10, 50).has_more(551), true);
    assert_eq!(BackendPaginationOptions::new(10, 50).has_more(400), false);
}

impl From<&BackendPaginationOptions> for gadget_database::prelude::PaginationOptions {
    fn from(options: &BackendPaginationOptions) -> Self {
        Self {
            page_number: options.page_number,
            page_size: options.page_size,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BackendRedirect {
    pub public_ref: String,
    pub alias: String,
    pub destination: String,
    pub visiable_to_all: bool,
    pub created_on: DateTime<Utc>,
    pub created_by: BackendRedirectUser,
}

impl From<&BackendRedirect> for DbRedirect {
    fn from(model: &BackendRedirect) -> Self {
        Self {
            public_ref: model.public_ref.to_owned(),
            alias: model.alias.to_owned(),
            destination: model.destination.to_owned(),
            created_on: model.created_on,
        }
    }
}

impl BackendRedirect {
    pub fn set_destination(&mut self, destination: &str) {
        self.destination = destination.to_string();
    }

    pub fn new(alias: &str, destination: &str, created_by: BackendRedirectUser) -> Self {
        BackendRedirect {
            public_ref: make_random_id(),
            alias: alias.to_string(),
            destination: destination.to_string(),
            created_on: Utc::now(),
            visiable_to_all: true,
            created_by,
        }
    }

    pub fn from_db_models(redirect: &DbRedirect, user: &DbUser) -> Self {
        BackendRedirect {
            public_ref: redirect.public_ref.to_owned(),
            alias: redirect.alias.to_owned(),
            destination: redirect.destination.to_owned(),
            created_on: redirect.created_on,
            visiable_to_all: true,
            created_by: user.into(),
        }
    }
}

impl From<&DbRedirectUserPair> for BackendRedirect {
    fn from(pair: &DbRedirectUserPair) -> Self {
        BackendRedirect {
            public_ref: pair.redirect.public_ref.to_owned(),
            alias: pair.redirect.alias.to_owned(),
            destination: pair.redirect.destination.to_owned(),
            created_on: pair.redirect.created_on.to_owned(),
            visiable_to_all: true,
            created_by: (&pair.user).into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendRedirectList {
    pub redirects: Vec<BackendRedirect>,
    pub total_count: u32,
    pub has_more: bool,
}

impl BackendRedirectList {
    pub fn from(redirects: Vec<BackendRedirect>, total_count: u32, has_more: bool) -> Self {
        Self {
            redirects,
            total_count,
            has_more,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BackendRedirectUser {
    pub name: String,
    pub external_id: String,
}

impl From<&DbUser> for BackendRedirectUser {
    fn from(user: &DbUser) -> Self {
        Self {
            name: user.name.to_owned(),
            external_id: user.external_id.to_owned(),
        }
    }
}

impl From<&BackendRedirectUser> for DbUser {
    fn from(user: &BackendRedirectUser) -> Self {
        Self {
            name: user.name.to_owned(),
            external_id: user.external_id.to_owned(),
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
