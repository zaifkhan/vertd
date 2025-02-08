use actix_web::{web, App, HttpServer};
use log::info;
use services::{download::download, upload::upload, websocket::websocket};

mod response;
mod services;

pub async fn start_http() -> anyhow::Result<()> {
    let server = HttpServer::new(|| {
        App::new().service(
            web::scope("/api")
                .service(upload)
                .service(download)
                // .route("/ws", web::get().to(websocket)),
                .service(websocket),
        )
    });
    info!("http server listening on 0.0.0.0:8080");
    server.bind("0.0.0.0:8080")?.run().await?;
    Ok(())
}
