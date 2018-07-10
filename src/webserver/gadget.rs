use iron::Handler;
use iron::prelude::*;
use iron::status;
use std::io::prelude::*;
use std::sync::Arc;

use json::JsonValue;

use super::super::datasource::{DataSource, DataSourceContainer};

pub struct GadgetGetRequestHandler {}

impl GadgetGetRequestHandler {
    pub fn new() -> Self {
        return GadgetGetRequestHandler {};
    }
}

impl Handler for GadgetGetRequestHandler {
    fn handle(&self, _: &mut Request) -> Result<Response, IronError> {
        Ok(Response::with((status::Ok, "GoGo Gadget Redirect!".to_string())))
    }
}

pub struct GadgetPostRequestHandler {
    datasource: Arc<DataSourceContainer>
}

impl GadgetPostRequestHandler {
    pub fn new(datasource: Arc<DataSourceContainer>) -> Self {
        return GadgetPostRequestHandler { datasource };
    }
}

impl Handler for GadgetPostRequestHandler {
    fn handle(&self, request: &mut Request) -> Result<Response, IronError> {
        let mut buffer = String::new();

        if let Err(err) = request.body.read_to_string(&mut buffer) {
            let response_json = object! {
                "error" => "Internal Error!",
                "message" => err.to_string()
            }.dump();
            return Ok(Response::with((status::InternalServerError, response_json)));
        }

        let buffer = match json::parse(&buffer) {
            Ok(val) => val,
            Err(err) => {
                let response_json = object! {
                    "error" => "Invalid Body!",
                    "message" => err.to_string()
                }.dump();
                return Ok(Response::with((status::BadRequest, response_json)));
            }
        };

        let alias = match extract_value(&buffer, "alias") {
            Ok(value) => value,
            Err(resp) => return Ok(resp)
        };
        
        let redirect = match extract_value(&buffer, "redirect") {
            Ok(value) => value,
            Err(resp) => return Ok(resp)
        };

        return match self.datasource.add_new_redirect(&alias, &redirect) {
            Ok(_) => Ok(Response::with(status::Created)),
            Err(err) => {
                let response_json = object! {
                    "error" => "Unable to store redirect!",
                    "message" => err.message
                }.dump();
                Ok(Response::with((status::InternalServerError, response_json)))
            }
        };
    }
}

fn extract_value(value: &JsonValue, field: &str) -> Result<String, Response> {
    if value[field].is_string() {
        return Ok(value[field].as_str().unwrap().to_string())
    }

    return if value[field].is_null() {
        let json_body = object! { 
            "error" => "Missing Field",
            "message" => format!("The required field {} was missing.", field)
        }.dump();
        Err(Response::with((status::InternalServerError, json_body)))
    } else {
        let json_body = object! { 
            "error" => "Invalid Type",
            "message" => format!("The field {} was supposed to be a string instead of {:?}", field, value[field])
        }.dump();
        Err(Response::with((status::BadRequest, json_body)))
    };
}
