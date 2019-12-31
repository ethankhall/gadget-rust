use actix_web::{HttpRequest, HttpResponse};
use mime_guess::from_path;
use actix_web::body::Body;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "public/"]
struct Asset;

pub async fn serve_embedded(req: HttpRequest) -> HttpResponse {
    let mut path: String = req.match_info().query("filename").parse().unwrap();

    if path == "" {
        path = "index.html".to_string();
    }

    let content = Asset::get(&path).unwrap();
    let body: Body = match content {
        Cow::Borrowed(bytes) => bytes.into(),
        Cow::Owned(bytes) => bytes.into(),
    };

    HttpResponse::Ok().content_type(from_path(path).first_or_octet_stream().as_ref()).body(body)
}