use eyre::Result;

pub mod util;

pub mod work_queue;

mod file_server;
pub mod responders {
    pub use crate::file_server::FileServer;
}

mod http_request;
pub use http_request::HttpRequest;

mod http_response;
pub use http_response::HttpResponse;

mod http_server;
pub use http_server::HttpServer;

pub trait Responder {
    fn handle_request(&self, request: &HttpRequest) -> Result<HttpResponse>;
}
