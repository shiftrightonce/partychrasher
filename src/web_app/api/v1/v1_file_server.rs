use std::sync::Arc;

use actix_web::{
    error::{ErrorNotFound, ErrorUnauthorized},
    get, web, HttpRequest, Responder, Scope,
};

use crate::{db::DbManager, entity::client::OutClientEntityDto, web_app::when_user};

pub(crate) fn register_routes(scope: Scope) -> Scope {
    scope.service(serve).service(serve_file)
}

#[get("/stream/{id}")]
pub(crate) async fn serve(
    id: web::Path<String>,
    req: HttpRequest,
) -> actix_web::Result<actix_files::NamedFile> {
    let (_, response) = when_user::<OutClientEntityDto>(&req).await;

    if response.is_some() {
        return Err(ErrorUnauthorized("Permission denied"));
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    let the_id = id.into_inner();
    if let Some(media) = db_manager.media_repo().find_media_by_track(&the_id).await {
        actix_files::NamedFile::open(media.path).map_err(Into::into)
    } else {
        Err(ErrorNotFound(format!(
            "file with ID: {:?} not found",
            the_id
        )))
    }
}

#[get("serve/{media_id}")]
pub(crate) async fn serve_file(req: HttpRequest, media_id: web::Path<String>) -> impl Responder {
    let (_, response) = when_user::<OutClientEntityDto>(&req).await;

    if response.is_some() {
        return Err(ErrorUnauthorized("Permission denied"));
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    if let Some(media) = db_manager.media_repo().find_by_id(media_id.as_str()).await {
        actix_files::NamedFile::open(media.path).map_err(Into::into)
    } else {
        Err(ErrorNotFound(format!(
            "file with ID: {:?} not found",
            media_id
        )))
    }
}
