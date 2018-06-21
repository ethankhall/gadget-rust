use std::sync::Arc;

use router::Router;
use iron::{Url, status};
use iron::prelude::*;
use iron::Handler;
use iron::modifiers::Redirect;

use super::super::datasource::{DataSource, DataSourceContainer};
use super::StringError;

pub struct RedirectRequestHandler {
    datasource: Arc<DataSourceContainer>
}

impl RedirectRequestHandler {
    pub fn new(container: DataSourceContainer) -> Self {
        return RedirectRequestHandler { datasource: Arc::new(container) }
    }
}

impl Handler for RedirectRequestHandler {
    fn handle(&self, req: &mut Request) -> Result<Response, IronError> {

        let ref path = req.extensions.get::<Router>()
            .unwrap().find("redirect").unwrap_or("");

        let path = format!("{}", path);

        return match self.datasource.retrieve_lookup(path.clone()) {
            Some(redirect) => {
                debug!("Found lookup ({:?}) to map to {}", path.clone(), redirect);
                let url = Url::parse(&redirect).unwrap();
                Ok(Response::with((status::Found, Redirect(url))))
            },
            None => Err(IronError::new(StringError(format!("Unknown redirect `{}`", path)), status::BadRequest))
        };
    }
}
