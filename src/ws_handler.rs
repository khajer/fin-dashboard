use actix_web::{Error, HttpRequest, HttpResponse, rt, web};
use actix_ws::{AggregatedMessage, Session};
use futures_util::StreamExt as _;
use std::sync::{Arc, Mutex};
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

struct BinancePrice {
    symbol: String,
    price: String,
}

pub async fn handle(
    req: HttpRequest,
    stream: web::Payload,
    stocklist: web::Data<Arc<Mutex<Vec<&'static str>>>>,
    dashboard_clients: web::Data<Arc<Mutex<Vec<Session>>>>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20)); // 2MB

    info!("Client connected from: {}", req.peer_addr().unwrap());

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    info!("recv : {}", text);
                    parse_login_text(
                        &text,
                        stocklist.clone(),
                        dashboard_clients.clone(),
                        session.clone(),
                    )
                    .await;
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

async fn parse_login_text(
    text: &str,
    stocklist: web::Data<Arc<Mutex<Vec<&'static str>>>>,
    dashboard_clients: web::Data<Arc<Mutex<Vec<Session>>>>,
    mut session: Session,
) -> Result<(), Error> {
    let usr = serde_json::from_str::<Username>(&text);
    match usr {
        Ok(u) => {
            let mut list = stocklist.lock().unwrap();
            if list.is_empty() {
                return Ok(());
            }
            let symbol = list.remove(0).to_string();
            drop(list);
            if u.username == "bot" {
                let log_resp = LoginResponse {
                    status: "success".to_string(),
                    cmd: symbol.clone(),
                };
                let txt_resp = serde_json::to_string(&log_resp).unwrap();
                session.text(txt_resp).await.unwrap();
                return Ok(());
            }

            if u.username == "dashboard" {
                let log_resp = LoginResponse {
                    status: "success".to_string(),
                    cmd: symbol.clone(),
                };
                let mut clients = dashboard_clients.lock().unwrap();
                clients.push(session.clone());

                let txt_resp = serde_json::to_string(&log_resp).unwrap();
                session.text(txt_resp).await.unwrap();
                return Ok(());
            }
            Err(actix_web::error::ErrorBadRequest("Invalid username"))
        }
        Err(_) => {
            info!("recv: {}", text);
            Err(actix_web::error::ErrorBadRequest("Invalid request format"))
        }
    }
}
