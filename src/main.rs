use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, Responder, get, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;

use tracing::info;
use tracing_subscriber::fmt;

fn setup_logging() {
    fmt()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .init();
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
                    session.text(text).await.unwrap();
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
            .service(web::resource("/ws").route(web::get().to(echo)))
    })
    .bind(host)?
    .run()
    .await
}
