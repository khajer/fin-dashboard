use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, Responder, get, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;

use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_subscriber::fmt;

#[derive(Debug, Serialize, Deserialize)]
struct Username {
    username: String,
}

fn setup_logging() {
    fmt()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .init();
}
#[derive(Serialize)]
struct HealthStatus {
    status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    status: String,
    cmd: String,
}

#[get("/health")]
async fn health_check() -> Result<impl Responder, Error> {
    let status = HealthStatus {
        status: "ok".to_string(),
        version: Some("1.0.0".to_string()),
    };
    Ok(web::Json(status))
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn echo(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
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
                    let usr = serde_json::from_str::<Username>(&text).unwrap();
                    if usr.username == "bot" {
                        let log_resp = LoginResponse {
                            status: "success".to_string(),
                            cmd: "BTCUSDT".to_string(),
                        };
                        let txt_resp = serde_json::to_string(&log_resp).unwrap();

                        session.text(txt_resp).await.unwrap();
                    }

                    // session.text("").await.unwrap();
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_logging();
    let host = "127.0.0.1:8080";
    info!("Run server at: http://{}", host);

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(health_check)
            .service(web::resource("/ws").route(web::get().to(echo)))
    })
    .bind(host)?
    .run()
    .await
}
