use std::path::{Path, PathBuf};
use std::sync::Arc;

use mime_guess::from_path;
use warp::http::header::CONTENT_TYPE;

#[derive(Clone)]
pub struct WebDirectory {
    path: Arc<String>,
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

        let dir = root_path
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        Some(WebDirectory {
            path: Arc::new(dir),
        })
    }

    fn get_path(&self) -> &str {
        &self.path
    }
}

pub async fn serve_embedded(
    path: warp::filters::path::Tail,
    web_dir: Arc<WebDirectory>,
) -> Result<warp::reply::Response, std::convert::Infallible> {
    let mut path = path.as_str();

    if path == "" {
        path = "index.html";
    }

    let mut file_path = PathBuf::from(web_dir.get_path()).join(path);

    if !file_path.exists() {
        file_path = PathBuf::from(web_dir.get_path()).join("index.html");
    }

    let file_path = match file_path.canonicalize() {
        Ok(f) => f,
        Err(e) => {
            warn!("Unable to find file {:?} because {:?}", file_path, e);
            return not_found();
        }
    };

    if file_path.strip_prefix(web_dir.get_path()).is_err() {
        warn!("Someone is doing something bad and trying to leaving the web path. Giving 404. Path was {:?}", file_path);
        return not_found();
    }

    let body = match std::fs::read(&file_path) {
        Ok(s) => s,
        Err(e) => {
            warn!("Unable to read {:?} because {:?}", file_path, e);
            return not_found();
        }
    };

    let body = hyper::Body::from(body);
    Ok(warp::http::Response::builder()
        .header(
            CONTENT_TYPE,
            from_path(file_path).first_or_octet_stream().as_ref(),
        )
        .body(body)
        .unwrap())
}

fn not_found() -> Result<warp::reply::Response, std::convert::Infallible> {
    Ok(warp::http::Response::builder()
        .status(warp::http::StatusCode::NOT_FOUND)
        .body(hyper::Body::default())
        .unwrap())
}
