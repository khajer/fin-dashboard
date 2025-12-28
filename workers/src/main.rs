use futures_util::stream::SplitSink;

use futures_util::{SinkExt, StreamExt};

use tokio::time::{Duration, sleep};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

#[derive(Debug, Deserialize)]
struct BinancePriceResponse {
    symbol: String,
    price: String,
}

#[derive(Debug, Serialize)]
struct Username {
    username: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    status: String,
    cmd: String,
}

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

    let (write_stream, mut read) = ws_stream.split();
    let mut write = Some(write_stream);

    // bot login
    let usr = Username {
        username: "bot".to_string(),
    };
    println!("Username: {:?}", usr);
    let message = serde_json::to_string(&usr).unwrap();

    if let Some(ref mut w) = write {
        if let Err(e) = w.send(message.into()).await {
            eprintln!("Failed to send message: {}", e);
            return;
        }
    }

    let mut logined = false;
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("recv : {}", text);
                if !logined {
                    let login_resp = serde_json::from_str::<LoginResponse>(&text);
                    if let Ok(resp) = login_resp {
                        if resp.status == "success" {
                            println!("Login successful");

                            if let Some(w) = write.take() {
                                println!("write is OK");
                                let _ = interval_func(w, resp.cmd).await;
                                // let _ = spawn_write(w, resp.cmd);
                            } else {
                                println!("write is None");
                            }
                            logined = true;
                        }
                    } else {
                        eprintln!("Failed to parse login response: {}", text);
                    }
                }

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
async fn interval_func(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    cmd: String,
) {
    tokio::spawn(async move {
        loop {
            let price = fetch_price(cmd.clone()).await;
            match price {
                Ok(val) => {
                    let msg = format!("{}:{}", val.symbol, val.price);
                    println!("{msg}");
                    if let Err(e) = write.send(msg.into()).await {
                        eprintln!("Failed to send message: {}", e);
                        break;
                    } else {
                        println!("Message sent successfully");
                    }
                }
                Err(_) => {
                    println!("cannot get data");
                }
            }
            sleep(Duration::from_millis(1000)).await;
        }
    });
}

async fn fetch_price(symbol: String) -> Result<BinancePriceResponse, reqwest::Error> {
    let client = Client::new();
    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={}",
        symbol
    );
    let response = client
        .get(url)
        .send()
        .await?
        .json::<BinancePriceResponse>()
        .await?;
    Ok(response)
}
