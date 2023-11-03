use std::collections::HashMap;

pub struct HttpResponse {
    pub headers: HashMap<String, String>,
    pub http_version: String,
    pub method: String,
    pub status: u16,
    pub status_text: String,
    pub body: Option<Vec<u8>>,
}

impl HttpResponse {
    pub fn new(version: &str, method: &str) -> Self {
        HttpResponse {
            headers: HashMap::new(),
            http_version: String::from(version),
            method: String::from(method),
            status: 0,
            status_text: String::new(),
            body: Option::None,
        }
    }

    pub fn bad_request(version: &str, method: &str) -> Self {
        let mut res = HttpResponse::new(version, method);
        res.status = 400u16;
        res.status_text = String::from("Bad Request");
        res.headers.insert(String::from("content-type"), String::from("text/plain; charset=utf-8"));
        res.headers.insert(String::from("content-length"), String::from("11"));
        res.body = Some("Bad Request".into());

        res
    }

    pub fn unauthorized(version: &str, method: &str) -> Self {
        let mut res = HttpResponse::new(version, method);
        res.status = 401u16;
        res.status_text = String::from("Unauthorized");
        res.headers.insert(String::from("content-type"), String::from("text/plain; charset=utf-8"));
        res.headers.insert(String::from("content-length"), String::from("12"));
        res.body = Some("Unauthorized".into());

        res
    }

    pub fn forbidden(version: &str, method: &str) -> Self {
        let mut res = HttpResponse::new(version, method);
        res.status = 403u16;
        res.status_text = String::from("Forbidden");
        res.headers.insert(String::from("content-type"), String::from("text/plain; charset=utf-8"));
        res.headers.insert(String::from("content-length"), String::from("9"));
        res.body = Some("Forbidden".into());

        res
    }

    pub fn not_found(version: &str, method: &str) -> Self {
        let mut res = HttpResponse::new(version, method);
        res.status = 404u16;
        res.status_text = String::from("Not Found");
        res.headers.insert(String::from("content-type"), String::from("text/plain; charset=utf-8"));
        res.headers.insert(String::from("content-length"), String::from("9"));
        res.body = Some("Not Found".into());

        res
    }

    pub fn server_error(version: &str, method: &str) -> Self {
        let mut res = HttpResponse::new(version, method);
        res.status = 500u16;
        res.status_text = String::from("Server Error");
        res.headers.insert(String::from("content-type"), String::from("text/plain; charset=utf-8"));
        res.headers.insert(String::from("content-length"), String::from("12"));
        res.body = Some("Server Error".into());

        res
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut out = format!("{} {} {}\r\n", self.http_version, self.status, self.status_text);

        for (key, val) in &self.headers {
            out.push_str(&format!("{key}: {val}\r\n"));
        }
        out.push_str("\r\n");


        let mut out: Vec<u8> = out.into();

        match &self.body {
            Some(b) => out.extend(b),
            None => (),
        };

        out
    }
}

