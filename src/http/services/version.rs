use actix_web::{get, Responder};
use rbtag::BuildInfo;

use crate::http::response::ApiResponse;

#[derive(BuildInfo)]
struct BuildTag;

#[get("/version")]
pub async fn version() -> impl Responder {
    ApiResponse::Success(BuildTag {}.get_build_commit())
}
