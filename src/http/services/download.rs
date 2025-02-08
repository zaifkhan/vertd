// get /download/{id} where id is Uuid

use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder, ResponseError};
use tokio::fs;
use uuid::Uuid;

use crate::{http::response::ApiResponse, state::APP_STATE};

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("job not found")]
    NotFound,
    #[error("invalid token")]
    InvalidToken,
}

impl ResponseError for DownloadError {
    fn error_response(&self) -> HttpResponse {
        let status = match self {
            DownloadError::NotFound => actix_web::http::StatusCode::NOT_FOUND,
            DownloadError::InvalidToken => actix_web::http::StatusCode::UNAUTHORIZED,
        };

        HttpResponse::build(status).json(ApiResponse::<()>::Error(self.to_string()))
    }
}

#[get("/download/{id}?token={token}")]
pub async fn download(
    req: HttpRequest,
    path: web::Path<(Uuid, String)>,
) -> Result<impl Responder, DownloadError> {
    let (id, token) = path.into_inner();
    let app_state = APP_STATE.lock().await;
    let job = app_state
        .jobs
        .get(&id)
        .ok_or(DownloadError::NotFound)?
        .clone();
    drop(app_state);

    if job.auth != token {
        return Err(DownloadError::InvalidToken);
    }

    let file_path = match job.to {
        Some(to) => format!("output/{}.{}", id, to),
        None => return Err(DownloadError::NotFound),
    };

    let bytes = fs::read(&file_path)
        .await
        .map_err(|_| DownloadError::NotFound)?;

    fs::remove_file(file_path)
        .await
        .map_err(|_| DownloadError::NotFound)?;

    Ok(HttpResponse::Ok().body(bytes))
}
