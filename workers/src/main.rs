use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};

use serde::{Deserialize, Serialize};
use tokio::time::{Duration, sleep};

use reqwest::Client;

use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use url::Url;
use clap::Command;

#[derive(Debug, Deserialize, Serialize)]
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

const HOST: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
    let _matches = Command::new("b0t").version("0.0.1").get_matches();
    let url_path = format!("ws://{}/ws", HOST);
    let url = Url::parse(&url_path).unwrap();

    loop {
        match connect_async(url.clone()).await {
            Ok(result) => {
                let (ws_stream, _response) = result;
                cmd_data_socket(ws_stream).await;
            }
            Err(e) => {
                eprintln!("Failed to connect: {}", e);
                sleep(Duration::from_millis(1000)).await;
            }
        }
    }
}

async fn cmd_data_socket(
    ws_stream: WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
) {
    let (write_stream, mut read) = ws_stream.split();
    let mut write = Some(write_stream);

    // bot login
    let usr = Username {
        username: "bot".to_string(),
    };

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
                                let _ = interval_func(w, resp.cmd).await;
                            } else {
                                println!("errors has happened!!!");
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
                    let msg = serde_json::to_string(&val).unwrap();
                    println!("send: {}", msg);
                    if let Err(e) = write.send(msg.into()).await {
                        eprintln!("Failed to send message: {}", e);
                        break;
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
