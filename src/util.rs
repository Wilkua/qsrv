use std::collections::HashMap;

pub fn parse_method(buf: &[u8]) -> String {
    let idx = buf.iter().position(|&i| i as char == ' ').unwrap_or(0);

    String::from_utf8(buf[..idx].to_vec()).unwrap()
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

pub fn parse_locator(buf: &[u8], start: usize) -> (String, usize) {
    if start >= buf.len() {
        return ("".to_string(), start);
    }

    let idx = start + buf[start..]
        .iter()
        .position(|&i| i as char == ' ')
        .unwrap_or(0usize);

    match String::from_utf8(buf[start..idx].to_vec()) {
        Ok(s) => (s, idx),
        Err(_) => (String::from("???"), idx),
    }
}

pub fn parse_version(buf: &[u8], start: usize) -> (String, usize) {
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

pub fn parse_headers(buf: &[u8], start: usize) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    let mut sbuf = &buf[start..];

    loop {
        let (header_name, buf) = parse_header_name(sbuf);
        if header_name.is_empty() {
            break;
        }

        let (header_value, buf) = parse_header_value(buf);

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

