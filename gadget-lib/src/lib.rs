mod backend;
mod error;

use crate::backend::prelude::*;
use api::ApiRedirect;
use prelude::{GadgetLibError, LibResult};
use std::path::PathBuf;
use tracing::{debug, warn};

pub mod prelude {
    pub use crate::backend::prelude::*;
    pub use crate::create_backend;
    pub use crate::error::GadgetLibError;
    pub use crate::{AliasRedirect, Redirect};

    pub type LibResult<T> = std::result::Result<T, GadgetLibError>;
}

pub mod api {

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct RedirectList {
        pub redirects: Vec<ApiRedirect>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct ApiRedirect {
        pub alias: String,
        pub destination: String,
        pub created_by: Option<UserDetails>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct UpdateRedirect {
        pub destination: String,
        pub created_by: Option<UserDetails>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct UserDetails {
        pub username: String,
    }

    impl From<crate::prelude::RedirectModel> for ApiRedirect {
        fn from(model: crate::prelude::RedirectModel) -> Self {
            ApiRedirect {
                alias: model.alias,
                destination: model.destination,
                created_by: model.created_by.map(|name| UserDetails {
                    username: name.clone(),
                }),
            }
        }
    }
}

pub fn create_backend<'a>(url: String) -> LibResult<Box<dyn Backend<'a>>> {
    if url.starts_with("file://") {
        let path = PathBuf::from(url);
        let json_backend = JsonBackend::new(path)?;
        Ok(Box::new(json_backend))
    } else if url.starts_with("memory://") {
        Ok(Box::new(InMemoryBackend::new(Default::default())))
    } else {
        Err(GadgetLibError::UnknownBackend(url))
    }
}

pub trait Redirect {
    fn get_destination(&self, input: &str) -> String;
    fn evaluate(&self, input: &str) -> String;
    fn matches(&self, alias: &str) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
struct DestPart {
    number_of_components: usize,
    dest: String,
}

impl DestPart {
    fn new(number_of_components: usize, dest: String) -> Self {
        DestPart {
            number_of_components,
            dest,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AliasRedirect {
    alias: String,
    destinations: Vec<DestPart>,
}

impl From<RedirectModel> for AliasRedirect {
    fn from(value: RedirectModel) -> Self {
        AliasRedirect::new(&value.alias, &value.destination)
    }
}
impl From<ApiRedirect> for AliasRedirect {
    fn from(value: ApiRedirect) -> Self {
        AliasRedirect::new(&value.alias, &value.destination)
    }
}

#[test]
fn do_test() {
    let split: Vec<_> = "http://google.com{/foo/$1{/bar/$2}}"
        .split(|x: char| x == '{' || x == '}')
        .collect();
    assert_eq!(
        vec!["http://google.com", "/foo/$1", "/bar/$2", "", ""],
        split
    );
}

impl AliasRedirect {
    pub fn new(alias: &str, destination: &str) -> Self {
        let alias = if !alias.starts_with('/') {
            format!("/{}", alias)
        } else {
            alias.to_owned()
        };

        let mut destinations = Vec::new();

        let mut parts: Vec<_> = destination.split(|x: char| x == '{' || x == '}').collect();

        let last_open = destination.rfind('{');
        let first_close = destination.find('}');

        let base = parts.remove(0);
        let mut number_of_components = 0;

        if last_open > first_close {
            warn!("Destination has mismatched params: `{}`", destination);
            destinations.push(DestPart::new(number_of_components, destination.to_owned()));
        } else if parts.is_empty() {
            destinations.push(DestPart::new(number_of_components, destination.to_owned()));
        } else if parts.len() % 2 != 0 {
            warn!("Destination has missmatched `{{` | `}}`: `{}`", destination);
            destinations.push(DestPart::new(number_of_components, destination.to_owned()));
        } else {
            let mut pre = "".to_owned();
            let mut post = "".to_owned();

            destinations.push(DestPart::new(number_of_components, base.to_owned()));
            number_of_components += 1;

            while !parts.is_empty() {
                let size = parts.len() - 1;
                post = format!("{}{}", parts.remove(size), post);
                pre = format!("{}{}", pre, parts.remove(0));

                destinations.push(DestPart::new(
                    number_of_components,
                    format!("{}{}{}", base, pre, post),
                ));
                number_of_components += 1;
            }
        }

        AliasRedirect {
            alias,
            destinations,
        }
    }
}

impl Redirect for AliasRedirect {
    #[tracing::instrument(skip(self))]
    fn get_destination(&self, input: &str) -> String {
        let parsed_input = match urlencoding::decode(input) {
            Ok(s) => s.to_string(),
            Err(_) => input.to_owned(),
        };

        debug!("Parsed Input: {}", parsed_input);

        let mut inputs: Vec<&str> = parsed_input.split(' ').collect();
        inputs.remove(0);
        self.evaluate(&inputs.join(" "))
    }

    #[tracing::instrument(skip(self))]
    fn evaluate(&self, input: &str) -> String {
        let parsed_input = match urlencoding::decode(input) {
            Ok(s) => s.to_string(),
            Err(_) => input.to_owned(),
        };

        debug!("Parsed Input: {}", parsed_input);
        let mut inputs: Vec<&str> = parsed_input.split(' ').collect();

        let part = if inputs.len() <= self.destinations.len() {
            match self.destinations.get(inputs.len()) {
                Some(dest) => dest,
                None => self.destinations.last().unwrap(),
            }
        } else {
            self.destinations.last().unwrap()
        };

        let mut destination = part.dest.clone();

        for i in 1..=part.number_of_components {
            destination = destination.replace(&format!("${}", i), inputs.remove(0));
        }

        if !inputs.is_empty() {
            destination = format!("{} {}", destination, inputs.join(" "));
        }

        destination
    }

    fn matches(&self, alias: &str) -> bool {
        self.alias == alias.to_lowercase()
    }
}

#[test]
fn long_url_with_spaces() {
    use urlencoding::encode;

    let alias = AliasRedirect::new("google", "https://duckduckgo.com/{?q=$1}");

    assert_eq!(
        "https://duckduckgo.com/?q=let me google that for you",
        &alias.get_destination("google let me google that for you")
    );

    let alias = AliasRedirect::new("google", "https://duckduckgo.com/{?q=$1}");

    assert_eq!(
        "https://duckduckgo.com/?q=let me google that for you",
        &alias.get_destination(&encode("google let me google that for you"))
    );
}

#[test]
fn with_just_query() {
    let alias = AliasRedirect::new("google", "https://duckduckgo.com/{?q=$1}");

    assert_eq!("https://duckduckgo.com/", &alias.get_destination("google"));
}
