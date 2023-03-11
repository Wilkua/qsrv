// use crate::{HttpResponse, HttpRequest, Responder};
use http_body_util::Full;
use hyper::{
    service::Service,
    body::{Incoming, Bytes},
    Request, Response,
};
use std::{
    error::Error,
    future::Future,
    io::ErrorKind,
    path::{Path, PathBuf},
    pin::Pin,
};
use tokio::fs;
use tracing::{error, trace};

fn mime_for_file_ext(path: &Path) -> String {
    let mime = match path.extension() {
        Some(e) => match e.to_str() {
            Some(s) => match s {
                "aac" => "audio/aac",
                "abw" => "application/x-abiword",
                "arc" => "application/x-freearc",
                "avif" => "image/avif",
                "avi" => "video/x-msvideo",
                "azw" => "application/vnd.amazon.ebook",
                "bmp" => "image/bmp",
                "bz" => "application/x-bzip",
                "bz2" => "application/x-bzip2",
                "cda" => "application/x-cdf",
                "csh" => "application/x-csh",
                "css" => "text/css",
                "csv" => "text/csv",
                "doc" => "application/msword",
                "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "eot" => "application/vnd.ms-fontobject",
                "epub" => "application/epub+zip",
                "gz" => "application/gzip",
                "gif" => "image/gif",
                "htm" | "html" => "text/html",
                "ico" => "image/vnd.microsoft.icon",
                "ics" => "text/calendar",
                "jar" => "application/java-archive",
                "jpg" | "jpeg" => "image/jpeg",
                "js" | "mjs" => "text/javascript",
                "json" => "application/json",
                "jsonld" => "application/ld+json",
                "mid" | "midi" => "audio/midi",
                "mp3" => "audio/mpeg",
                "mp4" => "video/mp4",
                "mpg" |"mpeg" => "video/mpeg",
                "mpkg" => "application/vnd.apple.installer+xml",
                "odp" => "application/vnd.oasis.opendocument.presentation",
                "ods" => "application/vnd.oasis.opendocument.spreadsheet",
                "odt" => "application/vnd.oasis.opendocument.text",
                "oga" | "ogg" => "audio/ogg",
                "ogv" => "video/ogg",
                "ogx" => "application/ogg",
                "opus" => "audio/opus",
                "otf" => "font/otf",
                "png" => "image/png",
                "pdf" => "application/pdf",
                "php" => "application/x-httpd-php",
                "ppt" => "application/vnd.ms-powerpoint",
                "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                "rar" => "application/vnd.rar",
                "rtf" => "application/rtf",
                "sh" => "application/x-sh",
                "svg" => "image/svg+xml",
                "tar" => "application/x-tar",
                "tif" | "tiff" => "image/tiff",
                "ts" => "video/mp2t",
                "ttf" => "font/ttf",
                "txt" => "text/plain",
                "vsd" => "application/vnd.visio",
                "wasm" => "application/wasm",
                "wav" => "audio/wav",
                "weba" => "audio/webm",
                "webm" => "video/webm",
                "webp" => "image/webp",
                "woff" => "font/woff",
                "woff2" => "font/woff2",
                "xhtml" => "application/xhtml+xml",
                "xls" => "application/vnd.ms-excel",
                "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "xml" => "application/xml",
                "xul" => "application/vnd.mozilla.xul+xml",
                "zip" => "application/zip",
                "7z" => "application/x-7z-compressed",
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

pub struct FileResolver {
    root_path: PathBuf,
}

impl FileResolver {
    pub fn new(root: &str) -> Result<Self, Box<dyn Error>> {
        let can_path = std::fs::canonicalize(Path::new(root))?;
        trace!("root canonical path: {:?}", can_path.as_os_str());

        Ok(FileResolver {
            root_path: can_path,
        })
    }
}

impl Service<Request<Incoming>> for FileResolver {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let root = self.root_path.clone();

        let mut working_path = PathBuf::from(&root);
            working_path.push(&req.uri().path()[1..]);
            if working_path.is_dir() {
                trace!("requested directory - serving index");
                working_path.push("index.html");
            }
            trace!("working request path: {:?}", working_path.as_os_str());

        Box::pin(async move {
            let working_path = match fs::canonicalize(working_path).await {
                Ok(p) => p,
                Err(e) => {
                    match e.kind() {
                        ErrorKind::NotFound => trace!("Failed to canonicalize path: Not found"),
                        _ => error!("Failed to canonicalize path: {}", e),
                    }
                    return Ok(Response::builder() .status(404) .body(Full::new(Bytes::from("Not found".to_string()))).unwrap());
                },
            };

            if !working_path.starts_with(&root) {
                return Ok(Response::builder().status(403).body(Full::new(Bytes::from("Forbidden".to_string()))).unwrap());
            }

            if let Ok(buf) = fs::read(&working_path).await {
                let mime = mime_for_file_ext(&working_path);

                let res = Response::builder().status(200)
                    .header("Content-Type", mime)
                    .body(Full::new(Bytes::from(buf)))
                    .unwrap();

                return Ok(res);
            }

            Ok(Response::builder().status(404).body(Full::new(Bytes::from("File output".to_string()))).unwrap())
        })
    }
}
