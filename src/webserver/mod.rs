use iron::prelude::*;
use router::Router;
use std::error::Error;
use std::fmt::{self, Debug};
use std::sync::Arc;

use super::datasource::DataSourceContainer;

mod redirect;
mod gadget;

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
}

pub fn exec_webserver(datasource: Arc<DataSourceContainer>) {
    let redirect_handler = redirect::RedirectRequestHandler::new(datasource);
    let gadget_handler = gadget::GadgetRequestHandler::new();

    let mut router = Router::new();
    router.get("/gadget", gadget_handler, "gadget_base");
    router.get("/:redirect", redirect_handler, "redirect");
    Iron::new(router).http("localhost:3000").unwrap();
}