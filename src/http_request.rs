use crate::util;
use std::{
    collections::HashMap,
    io::Read,
    net::TcpStream,
};

pub struct HttpRequest {
    pub headers: HashMap<String, String>,
    pub http_version: String,
    pub locator: String,    // Raw HTTP locator
    pub method: String,
    pub params: HashMap<String, String>,
    pub path: String,
}

impl HttpRequest {
    pub fn new(stream: &mut TcpStream) -> HttpRequest {
        let mut buf = [0u8; 2000];
        let _ = stream.read(&mut buf);

        let method = util::parse_method(&buf);
        let (locator, next) = util::parse_locator(&buf, method.len() + 1);
        let (version, next) = util::parse_version(&buf, next + 1);
        let headers = util::parse_headers(&buf, next + 1);

        let (path, params) = util::parse_path_components(&locator.as_bytes());

        HttpRequest {
            headers,
            http_version: version,
            locator,
            method,
            params,
            path,
        }
    }
}

