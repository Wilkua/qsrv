use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};

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

fn parse_method(buf: &[u8]) -> String {
    let mut idx = 0;
    loop {
        if buf[idx] as char == ' ' {
            break;
        }
        idx += 1;
        if idx >= buf.len() {
            break;
        }
    }

    return String::from_utf8(buf[..idx].to_vec()).unwrap();
}

pub fn parse_path_components(buf: &[u8]) -> (String, HashMap<String, String>) {
    let query = HashMap::new();
    let mut idx = 0;
    loop {
        if buf[idx] as char == '?' {
            break;
        }
        idx += 1;
        if buf.len() <= idx {
            break;
        }
    }

    let path = String::from_utf8(buf[..idx].to_vec()).unwrap();
    // TODO(william): Parse query parameters)

    (path, query)
}

fn parse_locator(buf: &[u8], start: usize) -> (String, usize) {
    let mut idx = start;

    loop {
        if buf[idx] as char == ' ' {
            break;
        }
        idx += 1;
        if buf.len() <= idx {
            break;
        }
    }

    match String::from_utf8(buf[start..idx].to_vec()) {
        Ok(s) => (s, idx),
        Err(_) => (String::from("???"), idx),
    }
}

fn parse_version(buf: &[u8], start: usize) -> (String, usize) {
    let mut idx = start;
    loop {
        if buf[idx] as char == '\r' {
            break;
        }
        idx += 1;
        if buf.len() <= idx {
            break;
        }
    }

    match String::from_utf8(buf[start..idx].to_vec()) {
        Ok(s) => (s, idx + 1),
        Err(_) => (String::from("???"), idx),
    }
}

fn parse_headers(buf: &[u8], start: usize) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    let mut sbuf = &buf[start..];

    loop {
        let (header_name, buf) = parse_header_name(&sbuf);
        if header_name == "" {
            break;
        }

        let (header_value, buf) = parse_header_value(&buf);

        headers.insert(header_name.to_lowercase(), header_value);
        sbuf = &buf;
    }

    headers
}

fn parse_header_name(buf: &[u8]) -> (String, &[u8]) {
    let mut idx = 0;
    loop {
        if buf[idx] as char == ':' {
            break;
        }
        if buf[idx] as char == '\r' {
            idx = 0;
            break;
        }
        idx += 1;
        if buf.len() <= idx {
            idx = 0;
            break;
        }
    }

    let header_name = match String::from_utf8(buf[..idx].to_vec()) {
        Ok(s) => s,
        Err(_) => String::from(""),
    };

    (header_name, &buf[idx + 2..])
}

fn parse_header_value(buf: &[u8]) -> (String, &[u8]) {
    let mut idx = 0;

    loop {
        if buf[idx] as char == '\r' {
            break;
        }
        idx += 1;
        if buf.len() <= idx {
            break;
        }
    }

    let header_value = match String::from_utf8(buf[..idx].to_vec()) {
        Ok(s) => s,
        Err(_) => String::from(""),
    };

    (header_value, &buf[idx + 2..])
}

fn respond_with_file(mut req: HttpRequest, path: &PathBuf) {
    if let Ok(buf) = fs::read(path) {
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

        let mut headers = HashMap::new();
        headers.insert("content-type", mime);
        headers.insert("content-length", buf.len().to_string());

        let mut stream = &req.stream;

        let _ = stream.write("HTTP/1.1 200 OK\r\n".as_bytes());
        for (key, val) in headers {
            let _ = stream.write(format!("{key}: {val}\r\n").as_bytes());
        }
        let _ = stream.write("\r\n".as_bytes());
        let _ = stream.write(&buf);

        println!("200 {} \"{}\" {} bytes", req.method, req.locator, buf.len());

        return;
    }

    let _ = req.stream.write("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".as_bytes());

    println!("404 {} \"{}\"", req.method, req.locator);
}

fn handle_request(mut stream: TcpStream) {
    let mut buf = [0u8; 2000];
    let _ = stream.read(&mut buf);

    let mut req = HttpRequest::new(stream);

    req.method = parse_method(&buf);
    let (loc, next) = parse_locator(&buf, req.method.len() + 1);
    req.locator = loc;
    let (version, next) = parse_version(&buf, next + 1);
    req.http_version = version;
    req.headers = parse_headers(&buf, next + 1);

    let (path, _params) = parse_path_components(&req.locator.as_bytes());
    req.path = path;

    let path = String::from(&req.path);
    let path = Path::new(&path[1..]);
    if path.exists() && path.is_file() {
        respond_with_file(req, &path.to_path_buf());
        return;
    } else if path.exists() && path.is_dir() {
        // let tries = ["index.html", "index.htm"];
        // for &try_file in tries {
        //     let try_path = path.with_file_name(&try_file);
        //     if try_path.exists() && try_path.is_file() {
        //         respond_with_file(stream, &try_path)
        //     }
        // }
    }

    let _ = req.stream.write(
        format!("{} 404 Not Found\r\nContent-Length: 9\r\nContent-Type: text/plain\r\n\r\nNot Found", req.http_version).as_bytes()
    );
}

fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = match TcpListener::bind(addr) {
        Ok(l) => l,
        Err(e) => panic!("{:?}", e),
    };

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                handle_request(s);
            },
            Err(e) => {
                println!("error: {:?}", e);
            }
        };
    }
}
