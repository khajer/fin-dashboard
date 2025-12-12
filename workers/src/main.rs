use futures_util::{SinkExt, StreamExt};
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

    let (mut write, mut read) = ws_stream.split();

    // Send a message
    let message = Message::Text("Hello, WebSocket!".into());
    if let Err(e) = write.send(message).await {
        eprintln!("Failed to send message: {}", e);
        return;
    }
    println!("Sent message: Hello, WebSocket!");

    // Receive messages
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

    // Close the connection
    if let Err(e) = write.send(Message::Close(None)).await {
        eprintln!("Failed to close connection: {}", e);
    } else {
        println!("Connection closed successfully");
    }
}
