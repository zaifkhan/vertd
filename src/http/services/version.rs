use actix_web::{get, Responder};
use rbtag::BuildInfo;

use crate::http::response::ApiResponse;

#[derive(BuildInfo)]
struct BuildTag;

#[get("/version")]
pub async fn version() -> impl Responder {
    let build_tag = BuildTag {}.get_build_commit();
    if build_tag.starts_with("-") {
        return ApiResponse::Success("latest");
    }
    ApiResponse::Success(build_tag)
}
