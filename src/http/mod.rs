use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use log::info;
use services::{download::download, upload::upload, websocket::websocket};

mod response;
mod services;

pub async fn start_http() -> anyhow::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .service(
                web::scope("/api")
                    .service(upload)
                    .service(download)
                    // .route("/ws", web::get().to(websocket)),
                    .service(websocket),
            )
    });
    info!("http server listening on 0.0.0.0:24153");
    server.bind("0.0.0.0:24153")?.run().await?;
    Ok(())
}
