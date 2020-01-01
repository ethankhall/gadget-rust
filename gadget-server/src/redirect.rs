use crate::backend::RedirectModel;

pub trait Redirect {
    fn get_destination(&self, input: &str) -> String;
    fn matches(&self, alias: &str) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AliasRedirect {
    alias: String,
    destinations: Vec<String>,
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
        let alias = if !alias.starts_with("/") {
            format!("/{}", alias)
        } else {
            alias
        };

        let mut destinations = Vec::new();

        let mut parts: Vec<_> = destination.split(|x: char| x == '{' || x == '}').collect();

        let last_open = destination.rfind("{");
        let first_close = destination.find("}");

        let base = parts.remove(0);

        if last_open > first_close {
            warn!("Destination has mismatched params: `{}`", destination);
            destinations.push(destination.clone());
        } else if parts.len() == 0 {
            destinations.push(destination.clone());
        } else if parts.len() % 2 != 0 {
            warn!("Destination has missmatched `{{` | `}}`: `{}`", destination);
            destinations.push(destination.clone());
        } else {
            let mut pre = s!("");
            let mut post = s!("");

            destinations.push(s!(base));

            while !parts.is_empty() {
                let size = parts.len() - 1;
                post = format!("{}{}", parts.remove(size), post);
                pre = format!("{}{}", pre, parts.remove(0));

                destinations.push(format!("{}{}{}", base, pre, post));
            }
        }

        AliasRedirect {
            alias,
            destinations,
        }
    }
}

impl Redirect for AliasRedirect {
    fn get_destination(&self, input: &str) -> String {
        let mut inputs: Vec<&str> = input.split(" ").collect();
        inputs.remove(0);
        let (size, destination) = if inputs.len() <= self.destinations.len() {
            (
                inputs.len(),
                s!(self.destinations.get(inputs.len()).unwrap()),
            )
        } else {
            (
                self.destinations.len(),
                format!(
                    "{} {}",
                    self.destinations.last().unwrap(),
                    inputs[inputs.len()..].join(" ")
                ),
            )
        };

        let mut destination = s!(destination);

        for i in 1..=size {
            destination = destination.replace(&format!("${}", i), inputs.remove(0));
        }

        return destination;
    }

    fn matches(&self, alias: &str) -> bool {
        self.alias == alias.to_lowercase()
    }
}
