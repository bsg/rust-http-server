use std::collections::HashMap;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::net::tcp::ReadHalf;

#[derive(Debug)]
pub struct HttpHeader {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug)]
pub enum HttpRequest {
    Get(HttpHeader),
    Post(HttpHeader),
}

pub enum HttpParseResult {
    Ok(HttpRequest),
    Err(String),
}

impl HttpRequest {
    pub async fn from_stream(
        stream: BufReader<ReadHalf<'_>>,
    ) -> (BufReader<ReadHalf<'_>>, HttpParseResult) {
        let mut header = HttpHeader {
            method: "".to_string(),
            uri: "".to_string(),
            version: "".to_string(),
            headers: HashMap::new(),
        };

        let mut lines = stream.lines();
        match lines.next_line().await {
            Ok(o) => match o {
                Some(line) => {
                    let mut it = line.split(|c: char| c == ' ');
                    match it.next() {
                        Some(s) => {
                            header.method = s.to_string();
                        }
                        None => return (lines.into_inner(), HttpParseResult::Err("".to_string())),
                    }
                    match it.next() {
                        Some(s) => {
                            header.uri = s.to_string();
                        }
                        None => return (lines.into_inner(), HttpParseResult::Err("".to_string())),
                    }
                    match it.next() {
                        Some(s) => {
                            header.version = s.to_string();
                        }
                        None => return (lines.into_inner(), HttpParseResult::Err("".to_string())),
                    }
                }
                None => return (lines.into_inner(), HttpParseResult::Err("".to_string())),
            },
            Err(_e) => {
                return (
                    lines.into_inner(),
                    HttpParseResult::Err("IO error".to_string()),
                );
            }
        }

        loop {
            match lines.next_line().await {
                Ok(o) => match o {
                    Some(line) => {
                        if line.is_empty() {
                            break;
                        } else {
                            let mut it = line.splitn(2, ": ");
                            match it.next() {
                                Some(k) => match it.next() {
                                    Some(v) => {
                                        header.headers.insert(k.to_string(), v.to_string());
                                    }
                                    None => {
                                        return (
                                            lines.into_inner(),
                                            HttpParseResult::Err("Malformed header".to_string()),
                                        );
                                    }
                                },
                                None => {
                                    return (
                                        lines.into_inner(),
                                        HttpParseResult::Err("Malformed header".to_string()),
                                    );
                                }
                            }
                        }
                    }
                    None => continue,
                },
                Err(_e) => {
                    return (
                        lines.into_inner(),
                        HttpParseResult::Err("IO error".to_string()),
                    );
                }
            }
        }

        if header.method == "GET" {
            (
                lines.into_inner(),
                HttpParseResult::Ok(HttpRequest::Get(header)),
            )
        } else if header.method == "POST" {
            (
                lines.into_inner(),
                HttpParseResult::Ok(HttpRequest::Post(header)),
            )
        } else {
            (
                lines.into_inner(),
                HttpParseResult::Err("Method not supported".to_string()),
            )
        }
    }
}
