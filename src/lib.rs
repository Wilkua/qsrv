use std::collections::HashMap;
use std::net::TcpStream;

pub mod file_server;
pub mod util;

/* pub struct WorkQueue<T>; */

/* impl<T> WorkQueue<T> { */
/*     pub fn new() -> WorkQueue<T> { */
/*         WorkQueue {} */
/*     } */
/* } */

pub struct HttpRequest {
    pub headers: HashMap<String, String>,
    pub http_version: String,
    pub locator: String,    // Raw HTTP locator
    pub method: String,
    pub params: HashMap<String, String>,
    pub path: String,
    pub stream: TcpStream,
}

impl HttpRequest {
    pub fn new(stream: TcpStream) -> HttpRequest {
        HttpRequest {
            headers: HashMap::new(),
            http_version: String::new(),
            locator: String::new(),
            method: String::new(),
            params: HashMap::new(),
            path: String::new(),
            stream
        }
    }
}

pub trait Responder {
    fn handle_request(&self, request: &HttpRequest) -> String;
}
