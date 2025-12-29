use actix_web::{App, Error, HttpResponse, HttpServer, Responder, get, web};

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
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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
            .service(web::resource("/ws").route(web::get().to(ws_handler::handle)))
    })
    .bind(host)?
    .run()
    .await
}
