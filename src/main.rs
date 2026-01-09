use std::sync::{Arc, Mutex};

use actix_web::{App, Error, HttpResponse, HttpServer, Responder, get, web};

use actix_ws::Session;
use serde::Serialize;
use tracing::info;
use tracing_subscriber::fmt;

#[derive(Serialize)]
struct HealthStatus {
    status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

mod ws_handler;

fn setup_logging() {
    fmt()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .init();
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
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/dashboard")]
async fn dashboard() -> impl Responder {
    let html_content = include_str!("../static/dashboard.html");
    HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_logging();
    let host = "127.0.0.1:8080";
    info!("Run server at: http://{}", host);

    let stocklist = Arc::new(Mutex::new(vec![
        "BTCUSDT",
        "ETHUSDT",
        "BNBUSDT",
        "SOLUSDT",
        "XRPUSDT",
        "DOTUSDT",
        "ADAUSDT",
        "TRXUSDT"
    ]));
    let dashboard_clients: Arc<Mutex<Vec<Session>>> = Arc::new(Mutex::new(Vec::new()));
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(stocklist.clone()))
            .app_data(web::Data::new(dashboard_clients.clone()))
            .service(health_check)
            .service(index)
            .service(dashboard)
            .service(web::resource("/ws").route(web::get().to(ws_handler::handle)))
    })
    .bind(host)?
    .run()
    .await
}
