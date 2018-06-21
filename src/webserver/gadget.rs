use iron::status;
use iron::prelude::*;
use iron::Handler;

pub struct GadgetRequestHandler {
}

impl GadgetRequestHandler {
    pub fn new() -> Self {
        return GadgetRequestHandler { };
    }
}

impl Handler for GadgetRequestHandler {
    fn handle(&self, _: &mut Request) -> Result<Response, IronError> {
        Ok(Response::with((status::Ok, "GoGo Gadget Redirect!".to_string())))
    }
}
