use crate::{HttpResponse, HttpRequest, Responder};
use eyre::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, trace};

fn mime_for_file_ext(path: &Path) -> String {
    let mime = match path.extension() {
        Some(e) => match e.to_str() {
            Some(s) => match s {
                "html" => "text/html",
                "css" => "text/css",
                "js" => "text/javascript",
                "svg" => "image/svg+xml",
                "jpg" => "image/jpeg",
                "jpeg" => "image/jpeg",
                "png" => "image/png",
                _ => "application/octet-stream",
            },
            None => "application/octet-stream",
        },
        None => "application/octet-stream",
    };
    let mime = if mime.starts_with("text/") {
        format!("{}; charset=utf-8", mime)
    } else {
        String::from(mime)
    };

    mime
}

pub struct FileServer {
    root_path: PathBuf,
}

impl FileServer {
    pub fn new(root: &str) -> Result<Self> {
        let can_path = std::fs::canonicalize(Path::new(root))?;
        trace!("root canonical path: {:?}", can_path.as_os_str());

        Ok(FileServer {
            root_path: can_path,
        })
    }
}

impl Responder for FileServer {
    fn handle_request(&self, req: &HttpRequest) -> Result<HttpResponse> {
        let mut working_path = PathBuf::from(&self.root_path);
        working_path.push(&req.path[1..]);
        if working_path.is_dir() {
            trace!("requested directory - serving index");
            working_path.push("index.html");
        }
        trace!("working request path: {:?}", working_path.as_os_str());
        working_path = match fs::canonicalize(working_path) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to canonicalize path: {}", e);
                return Ok(HttpResponse::not_found(&req.http_version, &req.method));
            },
        };

        if !working_path.starts_with(&self.root_path) {
            return Ok(HttpResponse::forbidden(&req.http_version, &req.method));
        }

        if let Ok(buf) = fs::read(&working_path) {
            let mime = mime_for_file_ext(&working_path);

            let mut res = HttpResponse::new(&req.http_version, &req.method);

            res.status = 200u16;
            res.status_text = String::from("OK");
            res.headers.insert(String::from("content-type"), mime);
            res.headers.insert(String::from("content-length"), buf.len().to_string());
            res.body = Some(buf);

            return Ok(res);
        }

        Ok(HttpResponse::not_found(&req.http_version, &req.method))
    }
}
