use std::convert::TryFrom;
use std::fmt;
use std::error::Error;

use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompiledConfigs {
    default_redirect: DefaultRedirect,
    alias_redirects: Vec<AliasRedirect>,
    direct_redirects: Vec<DirectRedirect>
}

impl CompiledConfigs {
    pub fn new(missing_redirect_destination: String, redirects: Vec<RedirectDefinition>) -> Self {
        let mut alias_redirects = Vec::new();
        let mut direct_redirects = Vec::new();
        for redirect in redirects {
            match CompiledRedirect::try_from(redirect.clone()) {
                Ok(CompiledRedirect::Alias(alias)) => alias_redirects.push(alias),
                Ok(CompiledRedirect::Direct(raw)) => direct_redirects.push(raw),
                Err(e) => {
                    warn!("DROPPED: Unable to process alias {} because {:?}", redirect.alias, e);
                }
            }
        }

        CompiledConfigs {
            default_redirect: DefaultRedirect::new(s!(missing_redirect_destination)),
            alias_redirects,
            direct_redirects
        }
    }

    pub fn find_redirect(&self, path: &str) -> String {
        let path = &path.to_lowercase();
        if let Some(dest) = self.direct_redirects.iter().find(|redirect| redirect.matches(path)) {
            return dest.get_destination(&path);
        }

        if let Some(dest) = self.alias_redirects.iter().find(|redirect| redirect.matches(path)) {
            return dest.get_destination(&path);
        }

        self.default_redirect.get_destination(&path)
    }
}

enum CompiledRedirect {
    Alias(AliasRedirect),
    Direct(DirectRedirect)
}

impl TryFrom<RedirectDefinition> for CompiledRedirect {
    type Error = ParseRedirectError;

    fn try_from(value: RedirectDefinition) -> Result<Self, Self::Error> {
        let split_alias: Vec<&str> = value.alias.split(":").collect();
        if split_alias.len() != 4 || !value.alias.starts_with("urn:gadget:") {
            return Err(ParseRedirectError::new(&value.destination));
        }

        match split_alias[2] {
            "alias" => Ok(CompiledRedirect::Alias(AliasRedirect::new(split_alias[3].to_lowercase(), s!(value.destination)))),
            "direct" => Ok(CompiledRedirect::Direct(DirectRedirect::new(split_alias[3].to_lowercase(), s!(value.destination)))),
            _ => {
                return Err(ParseRedirectError::new(&value.destination));
            }
        }
    }
}

#[derive(Debug)]
pub struct ParseRedirectError {
    body: String
}

impl ParseRedirectError {
    fn new(s: &str) -> Self {
        ParseRedirectError { body: s!(s) }
    }
}

impl Error for ParseRedirectError {
    fn description(&self) -> &str {
        "Unable to parse input string"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for ParseRedirectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to parse {}", self.body)
    }
}

trait Redirect {
    fn get_destination(&self, input: &str) -> String;

    fn matches(&self, alias: &str) -> bool;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DirectRedirect {
    pub alias: String,
    pub destination: String
}

impl DirectRedirect {
    fn new (alias: String, destination: String) -> Self {
        let alias = if !alias.starts_with("/") {
            format!("/{}", alias)
        } else {
            alias
        };

        DirectRedirect {
            alias,
            destination
        }
    }
}

impl Redirect for DirectRedirect {
    fn get_destination(&self, input: &str) -> String {
        let remainder = input.split_at(self.alias.len()).1;
        format!("{}{}", self.destination, remainder)
    }

    fn matches(&self, alias: &str) -> bool {
        self.alias == alias.to_lowercase()
    }
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AliasRedirect {
    pub alias: String,
    pub destinations: Vec<String>
}

#[test]
fn do_test() {
    let split: Vec<_> = "http://google.com{/foo/$1{/bar/$2}}".split(|x:char| x == '{' || x == '}').collect();
    assert_eq!(vec!["http://google.com", "/foo/$1", "/bar/$2", "", ""], split);
}

impl AliasRedirect {
    fn new (alias: String, destination: String) -> Self {
        let alias = if !alias.starts_with("/") {
            format!("/{}", alias)
        } else {
            alias
        };

        let mut destinations = Vec::new();

        let mut parts: Vec<_> = destination.split(|x:char| x == '{' || x == '}').collect();

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
            destinations
        }
    }
}

impl Redirect for AliasRedirect {
    fn get_destination(&self, input: &str) -> String {
        let mut inputs: Vec<&str> = input.split(" ").collect();
        inputs.remove(0);
        let (size, destination) = if inputs.len() <= self.destinations.len() {
            (inputs.len(), s!(self.destinations.get(inputs.len()).unwrap()))
        } else {
            (self.destinations.len(), format!("{} {}", self.destinations.last().unwrap(), inputs[inputs.len()..].join(" ")))
        };

        let mut destination = s!(destination);

        for i in 1..=size {
            destination = destination.replace(&format!("${}", i), inputs.remove(0));
        }

        return destination;
    }

    fn matches(&self, alias: &str) -> bool {
        self.alias.starts_with(&alias.to_lowercase())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DefaultRedirect {
    pub destination: String
}

impl DefaultRedirect {
    fn new (destination: String) -> Self {
        DefaultRedirect { destination }
    }
}

impl Redirect for DefaultRedirect {
    fn get_destination(&self, _input: &str) -> String {
        format!("{}", self.destination)
    }

    fn matches(&self, _alias: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_direct() {
        assert_eq!(DirectRedirect::new(s!("/foo"), s!("http://google.com")), make_direct_redirect("foo", "http://google.com"));
        assert_eq!("http://google.com/bar/baz", make_direct_redirect("foo", "http://google.com").get_destination("/foo/bar/baz"));
        assert_eq!("http://google.com", make_direct_redirect("foo", "http://google.com").get_destination("/foo"));
    }

    #[test]
    fn parse_alias() {
        let _ = simple_logger::init();

        assert_eq!(AliasRedirect::new(s!("/foo"), s!("http://google.com")), make_alias_redirect("foo", "http://google.com"));
        let alias = make_alias_redirect("foo", "http://google.com");
        assert_eq!("http://google.com", alias.get_destination("/foo"));

        let alias = make_alias_redirect("foo", "http://google.com{/foo/$1}");
        assert_eq!("http://google.com", alias.get_destination("/foo"));
        assert_eq!("http://google.com/foo/bar", alias.get_destination("/foo bar"));
    }

    fn make_direct_redirect(alias: &str, dest: &str) -> DirectRedirect {
        let redirect = RedirectDefinition {
            alias: format!("urn:gadget:direct:{}", alias),
            destination: format!("{}", dest)
        };

        match CompiledRedirect::try_from(redirect).unwrap() {
            CompiledRedirect::Direct(raw) => raw,
            _ => unimplemented!()
        }
    }

    fn make_alias_redirect(alias: &str, dest: &str) -> AliasRedirect {
        let redirect = RedirectDefinition {
            alias: format!("urn:gadget:alias:{}", alias),
            destination: format!("{}", dest)
        };

        match CompiledRedirect::try_from(redirect).unwrap() {
            CompiledRedirect::Alias(alias) => alias,
            _ => unimplemented!()
        }
    }
}