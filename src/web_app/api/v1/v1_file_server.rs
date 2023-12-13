use std::sync::Arc;

use actix_web::{
    error::{ErrorNotFound, ErrorUnauthorized},
    get, web, HttpRequest, Scope,
};

use crate::{db::DbManager, entity::client::OutClientEntityDto, web_app::when_user};

pub(crate) fn register_routes(scope: Scope) -> Scope {
    scope.service(serve)
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
    if let Some(track) = db_manager.track_repo().find_by_id(&the_id).await {
        actix_files::NamedFile::open(track.path).map_err(Into::into)
    } else {
        Err(ErrorNotFound(format!(
            "file with ID: {:?} not found",
            the_id
        )))
    }
}
