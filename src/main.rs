mod http;
use ascii::AsciiStr;
use ascii::AsciiString;
use chrono::offset::Utc;
use chrono::DateTime;
use http::HttpHeader;
use http::HttpParseResult;
use std::time::SystemTime;
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

async fn process(mut socket: TcpStream) {
    let mut chunk: [u8; 1024] = [0; 1024];
    let mut buf = AsciiString::with_capacity(1024 * 16);

    loop {
        socket.readable().await;
        match socket.try_read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buf.push_str(AsciiStr::from_ascii(&chunk[0..n]).unwrap());
                match http::HttpRequest::parse(&buf.as_str()) {
                    HttpParseResult::Ok(header) => match header {
                        http::HttpRequest::GET(header) => {
                            let header = dbg!(header);
                            socket = respond_time(socket, header);
                            buf.clear();
                        }
                        http::HttpRequest::POST(_) => (),
                    },
                    HttpParseResult::Incomplete => continue,
                    HttpParseResult::Err(e) => {
                        println!("{e:?}");
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("{e:?}");
                break;
            }
        }
    }
}

fn respond_time(socket: TcpStream, header: HttpHeader) -> TcpStream {
    let now = SystemTime::now();
    let datetime: DateTime<Utc> = now.into();
    let mut content = datetime.format("%d/%m/%Y %T").to_string();
    content.push_str(format!("\n\n{:#?}", header).as_str());
    let response = format!(
        "HTTP 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        content.len(),
        content
    );
    socket.try_write(response.as_bytes()).unwrap();
    socket
}
