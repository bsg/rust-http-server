mod http;
use std::time::SystemTime;
use chrono::offset::Utc;
use chrono::DateTime;
use ascii::AsciiStr;
use ascii::AsciiString;
use http::HttpParseResult;
use tokio::io;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    println!("Debug enabled");

    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("Listening");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        println!("Accepted");

        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    let mut chunk: [u8; 1024] = [0; 1024];
    let mut buf = AsciiString::with_capacity(1024 * 16);

    loop {
        socket.readable().await;
        match socket.try_read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buf.push_str(AsciiStr::from_ascii(&chunk[0..n]).unwrap());
                match http::HttpRequest::parse(&buf.as_str()) {
                    HttpParseResult::Ok(header) => {
                        match header {
                            http::HttpRequest::GET(header) => {
                                dbg!(header);
                                let now = SystemTime::now();
                                let datetime: DateTime<Utc> = now.into();
                                let content = datetime.format("%d/%m/%Y %T").to_string();
                                let response = format!("HTTP 200 OK\r\nContent-Length: {}\r\n\r\n{}", content.len(), content);
                                socket.try_write(response.as_bytes()).unwrap();
                                buf.clear();
                            }
                            http::HttpRequest::POST(_) => (),
                        }
                    },
                    HttpParseResult::Incomplete => continue,
                    HttpParseResult::Err(e) => {
                        println!("{e:?}");
                    },
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                break;
            }
        }
        
    }
}
