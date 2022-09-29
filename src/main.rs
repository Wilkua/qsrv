use eyre::Result;
use qsrv::{file_server::FileServer, HttpRequest, Responder};
use qsrv::util::{
    parse_headers,
    parse_locator,
    parse_method,
    parse_path_components,
    parse_version,
};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;

pub fn respond_with_file(mut req: HttpRequest, path: &PathBuf) {
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

fn handle_request(mut stream: TcpStream) -> Result<()> {
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
        let responder: FileServer = FileServer::new(".")?;
        let output = responder.handle_request(&req);
        println!("{output}");
        respond_with_file(req, &path.to_path_buf());

        return Ok(());
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

    Ok(())
}

/* enum Work { */
/*     Quit, */
/*     Request(HttpRequest), */
/* } */

fn main() {
    /* let mut queue: WorkQueue<Work> = WorkQueue::new(4, || { */
    /* }); */

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = match TcpListener::bind(addr) {
        Ok(l) => l,
        Err(e) => panic!("{:?}", e),
    };

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(|| {
                    handle_request(s);
                });
            },
            Err(e) => {
                println!("error: {:?}", e);
            }
        };
    }
}
