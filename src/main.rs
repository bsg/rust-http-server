mod http;
use chrono::offset::Utc;
use chrono::DateTime;
use http::HttpHeader;
use http::HttpParseResult;
use http::HttpRequest;
use std::time::SystemTime;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::net::tcp::ReadHalf;
use tokio::net::tcp::WriteHalf;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::runtime;

fn main() {
    #[cfg(debug_assertions)]
    println!("Debug enabled");

    //let rt = runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
        println!("Listening");

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            println!("Accepted");

            tokio::spawn(async move {
                process(socket).await;
            });
        }
    });
}

async fn process(mut socket: TcpStream) {
    let (socket_rx, socket_tx) = socket.split();
    let mut rx = BufReader::new(socket_rx);
    let mut tx = BufWriter::new(socket_tx);

    loop {
        let request;
        (rx, request) = HttpRequest::from_stream(rx).await;
        match request {
            HttpParseResult::Ok(request) => match request {
                HttpRequest::GET(header) => {
                    println!("GET {}", header.uri);
                    (rx, tx) = respond_time(rx, tx, header).await;
                    tx.flush().await;
                }
                HttpRequest::POST(header) => {
                    println!("POST {}", header.uri);
                }
            },
            HttpParseResult::Err(e) => {
                println!("{e:?}");
                break;
            }
        }
    }
}

async fn respond_time<'a>(
    rx: BufReader<ReadHalf<'a>>,
    mut tx: BufWriter<WriteHalf<'a>>,
    header: HttpHeader,
) -> (BufReader<ReadHalf<'a>>, BufWriter<WriteHalf<'a>>) {
    let now = SystemTime::now();
    let datetime: DateTime<Utc> = now.into();
    let mut content = datetime.format("%d/%m/%Y %T UTC").to_string();
    content.push_str(format!("\n\n{:#?}", header).as_str());
    let response = format!(
        "HTTP 200 OK\r\nConnection: keep-alive\r\nContent-Length: {}\r\n\r\n{}",
        content.len(),
        content
    );
    tx.write_all(response.as_bytes()).await;
    (rx, tx)
}
