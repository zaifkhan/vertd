use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use log::info;
use services::{download::download, upload::upload, version::version, websocket::websocket};
use crate::http::auth::Authentication;

mod auth;
mod response;
mod services;

pub async fn start_http(auth_token: String) -> anyhow::Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(web::Data::new(auth_token.clone()))
            .service(
                web::scope("/api")
                    .service(version) // Publicly accessible at /api/version
                    // All services below this wrapper are protected
                    .service(
                        web::scope("") // Create a sub-scope for auth
                            .wrap(Authentication)
                            .service(upload)
                            .service(download)
                            .service(websocket),
                    )
            )
    });
    let port = std::env::var("PORT").unwrap_or_else(|_| "24153".to_string());
    if !port.chars().all(char::is_numeric) {
        anyhow::bail!("PORT must be a number");
    }
    let ip = format!("0.0.0.0:{}", port);
    info!("http server listening on {}", ip);
    server.bind(ip)?.run().await?;
    Ok(())
}