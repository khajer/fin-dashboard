use actix_web::{Error, HttpRequest, HttpResponse, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
use tracing::info;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct LoginResponse {
    status: String,
    cmd: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Username {
    username: String,
}

pub async fn handle(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20)); // 2MB

    info!("Client connected from: {}", req.peer_addr().unwrap());

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    println!("recv : {}", text);

                    let usr = serde_json::from_str::<Username>(&text);
                    match usr {
                        Ok(u) => {
                            if u.username == "bot" {
                                let log_resp = LoginResponse {
                                    status: "success".to_string(),
                                    cmd: "BTCUSDT".to_string(),
                                };
                                let txt_resp = serde_json::to_string(&log_resp).unwrap();

                                session.text(txt_resp).await.unwrap();
                            }
                        }
                        Err(_) => {
                            println!("recv: {}", text);
                        }
                    }
                }
                Ok(AggregatedMessage::Binary(bin)) => {
                    session.binary(bin).await.unwrap();
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }

                _ => {}
            }
        }
    });

    Ok(res)
}
