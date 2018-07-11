use iron::Request;
use iron::IronResult;
use iron::Response;
use iron::status;
use iron::mime::Mime;
use iron::AroundMiddleware;
use iron::Handler;

use webserver::Asset;

pub(crate) struct Custom404;

struct Custom404Handler<H: Handler> {
    handler: H
}

impl<H: Handler> Handler for Custom404Handler<H> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let res = self.handler.handle(req);
        if let Ok(response) = res.as_ref() {
            if response.status == Some(status::NotFound) {
                let content_type = "text/html".parse::<Mime>().unwrap();
                let response = Response::with(
                    (content_type, status::NotFound, Asset::get("404.html").unwrap()));
                return Ok(response);
            }
        }
        return res;
    }
}

impl AroundMiddleware for Custom404 {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(Custom404Handler { handler })
    }
}