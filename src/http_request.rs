use crate::util;
use std::{
    collections::HashMap,
    io::Read,
};
use eyre::Error;
use tracing::debug;

pub struct HttpRequest {
    pub headers: HashMap<String, String>,
    pub http_version: String,
    pub locator: String,    // Raw HTTP locator
    pub method: String,
    pub params: HashMap<String, String>,
    pub path: String,
}

impl HttpRequest {
    pub fn new(stream: &mut impl Read) -> Result<HttpRequest, Error> {
        let mut buf = [0u8; 2000];
        let bytes_read = stream.read(&mut buf)?;
        debug!("read {} bytes", bytes_read);

        if bytes_read == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Read 0 bytes").into());
        }

        let method = util::parse_method(&buf);
        debug!("method = {:?}", method);
        let (locator, next) = util::parse_locator(&buf, method.len() + 1);
        debug!("locator = {:?}, next = {:?}", locator, next);
        let (version, next) = util::parse_version(&buf, next + 1);
        debug!("version = {:?}, next = {:?}", version, next);
        let headers = util::parse_headers(&buf, next + 1);
        debug!("headers = {:?}", headers);

        let (path, params) = util::parse_path_components(locator.as_bytes());
        debug!("path = {:?}, params = {:?}", path, params);

        Ok(HttpRequest {
            headers,
            http_version: version,
            locator,
            method,
            params,
            path,
        })
    }
}

