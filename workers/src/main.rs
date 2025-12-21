use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse("ws://127.0.0.1:8080/ws").unwrap();
    println!("Connecting to: {}", url);

    let (ws_stream, response) = match connect_async(url).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return;
        }
    };

    for (ref header, ref value) in response.headers() {
        println!("* {}: {:?}", header, value);
    }

    let (mut write, read) = ws_stream.split();

    // greeting message
    let message = Message::Text("Hello, WebSocket!".into());
    if let Err(e) = write.send(message).await {
        eprintln!("Failed to send message: {}", e);
        return;
    }

    let _ = spawn_write_task(write);
    let _ = spawn_read_task(read);
}

fn spawn_write_task(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let message = Message::Text("Hello, WebSocket!".into());
        if let Err(e) = write.send(message).await {
            eprintln!("Failed to send message: {}", e);
        } else {
            println!("Message sent successfully");
        }
    })
}

fn spawn_read_task(
    mut read: futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // receeive looping
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received text: {}", text);
                    break;
                }
                Ok(Message::Binary(bin)) => {
                    println!("Received binary data: {:?}", bin);
                    break;
                }
                Ok(Message::Close(_)) => {
                    println!("Received close message");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    println!("Received ping: {:?}", data);
                }
                Ok(Message::Pong(data)) => {
                    println!("Received pong: {:?}", data);
                }
                Ok(Message::Frame(_)) => {
                    // Raw frame, continue processing
                    continue;
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            }
        }
    })
}
