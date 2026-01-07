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

#[derive(Serialize, Deserialize)]
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
                    let parse_login = parse_login_text(
                        &text,
                        stocklist.clone(),
                        dashboard_clients.clone(),
                        session.clone(),
                    )
                    .await;

                    match parse_login {
                        Ok(_) => {}
                        Err(_err) => {
                            parse_command(&text, dashboard_clients.clone()).await;
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

pub async fn parse_login_text(
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

pub async fn parse_command(text: &str, dashboard_clients: web::Data<Arc<Mutex<Vec<Session>>>>) {
    match serde_json::from_str::<BinancePrice>(text) {
        Ok(data) => {
            let mut clients = dashboard_clients.lock().unwrap();
            for client in clients.iter_mut() {
                let msg = serde_json::to_string(&data).unwrap();
                info!("send: {}", msg);
                client.text(msg).await.unwrap();
            }
        }
        Err(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_valid_json_bot_username() {
        let text = r#"{"username": "bot"}"#;
        let result = serde_json::from_str::<Username>(text);
        assert!(result.is_ok());
        let username = result.unwrap();
        assert_eq!(username.username, "bot");
    }

    #[test]
    fn test_valid_json_dashboard_username() {
        let text = r#"{"username": "dashboard"}"#;
        let result = serde_json::from_str::<Username>(text);
        assert!(result.is_ok());
        let username = result.unwrap();
        assert_eq!(username.username, "dashboard");
    }

    #[test]
    fn test_invalid_json() {
        let text = r#"{"invalid": "json"}"#;
        let result = serde_json::from_str::<Username>(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_json() {
        let text = r#"{"username": bot"#;
        let result = serde_json::from_str::<Username>(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_json() {
        let text = r#"{}"#;
        let result = serde_json::from_str::<Username>(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_username() {
        let text = r#"{"username": "invalid"}"#;
        let result = serde_json::from_str::<Username>(text);
        assert!(result.is_ok());
        let username = result.unwrap();
        assert!(!["bot", "dashboard"].contains(&username.username.as_str()));
    }

    #[test]
    fn test_login_response_serialization() {
        let response = LoginResponse {
            status: "success".to_string(),
            cmd: "BTCUSDT".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("BTCUSDT"));
    }

    #[test]
    fn test_stocklist_initialization() {
        let stocklist: Arc<Mutex<Vec<&str>>> =
            Arc::new(Mutex::new(vec!["BTCUSDT", "ETHUSDT", "SOLBTC", "BNBUSDT"]));
        let list = stocklist.lock().unwrap();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0], "BTCUSDT");
        assert_eq!(list[1], "ETHUSDT");
    }

    #[test]
    fn test_stocklist_remove() {
        let stocklist: Arc<Mutex<Vec<&str>>> =
            Arc::new(Mutex::new(vec!["BTCUSDT", "ETHUSDT", "SOLBTC", "BNBUSDT"]));
        {
            let mut list = stocklist.lock().unwrap();
            let symbol = list.remove(0);
            assert_eq!(symbol, "BTCUSDT");
            assert_eq!(list.len(), 3);
        }
    }

    #[test]
    fn test_dashboard_clients_initialization() {
        let clients: Arc<Mutex<Vec<Session>>> = Arc::new(Mutex::new(Vec::new()));
        let client_list = clients.lock().unwrap();
        assert_eq!(client_list.len(), 0);
    }

    #[test]
    fn test_username_deserialize_special_chars() {
        let text = r#"{"username": "bot-user_123"}"#;
        let result = serde_json::from_str::<Username>(text);
        assert!(result.is_ok());
        let username = result.unwrap();
        assert_eq!(username.username, "bot-user_123");
    }

    #[test]
    fn test_username_case_sensitive() {
        let text = r#"{"username": "Bot"}"#;
        let result = serde_json::from_str::<Username>(text);
        assert!(result.is_ok());
        let username = result.unwrap();
        assert_eq!(username.username, "Bot");
        assert_ne!(username.username, "bot");
    }
}
