use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpHeader {
    pub length: usize,
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug)]
pub enum HttpRequest {
    GET(HttpHeader),
    POST(HttpHeader),
}

pub enum HttpParseResult {
    Ok(HttpRequest),
    Incomplete,
    Err(String),
}

impl HttpRequest {
    pub fn parse(buffer: &str) -> HttpParseResult {
        match buffer.find("\r\n\r\n") {
            Some(len) => {
                let mut header = HttpHeader {
                    method: "".to_string(),
                    length: len + 4,
                    uri: "".to_string(),
                    version: "".to_string(),
                    headers: HashMap::new(),
                };

                let mut lines = buffer.lines();
                match lines.next() {
                    Some(line) => {
                        let mut it = line.split(|c: char| c == ' ');
                        match it.next() {
                            Some(s) => {
                                header.method = s.to_string();
                            }
                            None => return HttpParseResult::Err("Malformed request".to_string()),
                        }
                        match it.next() {
                            Some(s) => {
                                header.uri = s.to_string();
                            }
                            None => return HttpParseResult::Err("Malformed request".to_string()),
                        }
                        match it.next() {
                            Some(s) => {
                                header.version = s.to_string();
                            }
                            None => return HttpParseResult::Err("Malformed request".to_string()),
                        }
                    }
                    None => return HttpParseResult::Err("Malformed request".to_string()),
                }

                for line in lines {
                    let key: String;
                    let value: String;

                    let mut it = line.split(": ");
                    match it.next() {
                        Some(s) => {
                            key = s.to_string();
                        }
                        None => {
                            continue;
                        }
                    }
                    match it.next() {
                        Some(s) => {
                            value = s.to_string();
                        }
                        None => {
                            continue;
                        }
                    }

                    header.headers.insert(key, value);
                }

                if header.method == "GET" {
                    HttpParseResult::Ok(HttpRequest::GET(header))
                } else if header.method == "POST" {
                    HttpParseResult::Ok(HttpRequest::POST(header))
                } else {
                    HttpParseResult::Err("Incorrect request method".to_string())
                }
            }
            None => HttpParseResult::Incomplete,
        }
    }
}
