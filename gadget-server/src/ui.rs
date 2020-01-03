use actix_web::{HttpRequest, HttpResponse, web, body::Body};
use mime_guess::from_path;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct WebDirectory {
    path: Arc<String>
}

impl WebDirectory {
    pub fn new(dir: String) -> Option<Self> {
        let root_path = Path::new(&dir);
        if !root_path.exists() {
            error!("UI Path {:?} does not exist", dir);
            return None;
        }

        let index_html = root_path.join("index.html");
        if !index_html.exists() {
            error!("UI Path {:?}/index.html must exist", dir);
            return None;
        }

        let dir = root_path.canonicalize().unwrap().to_str().unwrap().to_string();
        Some(WebDirectory { path: Arc::new(dir) })
    }

    fn get_path(&self) -> &str {
        &self.path
    }
}

pub async fn serve_embedded(req: HttpRequest, web_dir: web::Data<WebDirectory>) -> HttpResponse {
    let mut path: String = req.match_info().query("filename").parse().unwrap();

    if path == "" {
        path = "index.html".to_string();
    }

    let mut file_path = PathBuf::from(web_dir.get_path()).join(path);

    if !file_path.exists() {
        file_path = PathBuf::from(web_dir.get_path()).join("index.html");
    }

    let file_path = match file_path.canonicalize() {
        Ok(f) => f,
        Err(e) => {
            warn!("Unable to find file {:?} because {:?}", file_path, e);
            return HttpResponse::NotFound().finish();
        }
    };

    if file_path.strip_prefix(web_dir.get_path()).is_err() {
        warn!("Someone is doing something bad and trying to leaving the web path. Giving 404. Path was {:?}", file_path);
        return HttpResponse::NotFound().finish();
    }

    let body = match std::fs::read(&file_path) {
        Ok(s) => s,
        Err(e) => {
            warn!("Unable to read {:?} because {:?}", file_path, e);
            return HttpResponse::NotFound().finish();
        }
    };

    let body = Body::from(body);

    HttpResponse::Ok().content_type(from_path(file_path).first_or_octet_stream().as_ref()).body(body)
}