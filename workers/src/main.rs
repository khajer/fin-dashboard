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

    let (ws_stream, _response) = match connect_async(url).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            return;
        }
    };

    // for (ref header, ref value) in response.headers() {
    //     println!("* {}: {:?}", header, value);
    // }

    let (mut write, mut read) = ws_stream.split();

    // greeting message
    let message = Message::Text("Hi, WebSocket!".into());
    if let Err(e) = write.send(message).await {
        eprintln!("Failed to send message: {}", e);
        return;
    }

    let _ = spawn_write(write).await;

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text: {}", text);
                if text == "c" {
                    println!("Received exit command 'c', stopping loop...");
                    break;
                }
            }
            Ok(Message::Binary(bin)) => {
                println!("Received binary data: {:?}", bin);
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
}

async fn spawn_write(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            let message = Message::Text("test".into());
            println!("Sending message: test");
            if let Err(e) = write.send(message).await {
                eprintln!("Failed to send message: {}", e);
                break;
            } else {
                println!("Message sent successfully");
            }
        }
    })
}
