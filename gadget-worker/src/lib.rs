use serde_json::json;
use worker::*;

mod utils;
mod storage;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let url = req.url()?;

    let path = url.path().replace("%20", " ");

    let redirect_ref: Vec<&str> = path.split(' ').collect();
    let redirect_ref = match redirect_ref.first() {
        None => {
            return Ok(warp::http::Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, "/_gadget/ui")
                .body(hyper::Body::empty())
                .unwrap());
        }
        Some(value) => value,
    };

    
}
