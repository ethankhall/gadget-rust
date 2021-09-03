use crate::backend::RedirectModel;
use tracing::{debug, warn};

pub trait Redirect {
    fn get_destination(&self, input: &str) -> String;
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
        AliasRedirect::new(value.alias, value.destination)
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
    fn new(alias: String, destination: String) -> Self {
        let alias = if !alias.starts_with('/') {
            format!("/{}", alias)
        } else {
            alias
        };

        let mut destinations = Vec::new();

        let mut parts: Vec<_> = destination.split(|x: char| x == '{' || x == '}').collect();

        let last_open = destination.rfind('{');
        let first_close = destination.find('}');

        let base = parts.remove(0);
        let mut number_of_components = 0;

        if last_open > first_close {
            warn!("Destination has mismatched params: `{}`", destination);
            destinations.push(DestPart::new(number_of_components, destination.clone()));
        } else if parts.is_empty() {
            destinations.push(DestPart::new(number_of_components, destination.clone()));
        } else if parts.len() % 2 != 0 {
            warn!("Destination has missmatched `{{` | `}}`: `{}`", destination);
            destinations.push(DestPart::new(number_of_components, destination.clone()));
        } else {
            let mut pre = s!("");
            let mut post = s!("");

            destinations.push(DestPart::new(number_of_components, s!(base)));
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

    let alias = AliasRedirect::new(s!("google"), s!("https://duckduckgo.com/{?q=$1}"));

    assert_eq!(
        "https://duckduckgo.com/?q=let me google that for you",
        &alias.get_destination("google let me google that for you")
    );

    let alias = AliasRedirect::new(s!("google"), s!("https://duckduckgo.com/{?q=$1}"));

    assert_eq!(
        "https://duckduckgo.com/?q=let me google that for you",
        &alias.get_destination(&encode("google let me google that for you"))
    );
}

#[test]
fn with_just_query() {
    let alias = AliasRedirect::new(s!("google"), s!("https://duckduckgo.com/{?q=$1}"));

    assert_eq!("https://duckduckgo.com/", &alias.get_destination("google"));
}
