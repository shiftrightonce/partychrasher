use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpRequest, Responder, Scope};

use crate::{
    db::{DbManager, PaginatedResult, Paginator},
    entity::playlist::{InPlaylistEntityDto, OutPlaylistEntityDto},
    web_app::{api_response::ApiResponse, when_admin, when_user},
};

pub(crate) fn register_routes(scope: Scope) -> Scope {
    scope
        .service(get_playlists)
        .service(get_default)
        .service(get_a_playlist)
        .service(create)
        .service(update)
        .service(delete)
}

#[get("/playlists")]
async fn get_playlists(req: HttpRequest) -> impl Responder {
    let (_, response) = when_user::<OutPlaylistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let mut paginator = Paginator::try_from(&req).unwrap();
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    PaginatedResult::<Vec<OutPlaylistEntityDto>>::new(
        db_manager
            .playlist_repo()
            .paginate(&mut paginator)
            .await
            .into_iter()
            .map(OutPlaylistEntityDto::from)
            .collect::<Vec<OutPlaylistEntityDto>>(),
        &paginator,
    )
    .into_response()
}

#[get("/playlists/{id}")]
async fn get_a_playlist(req: HttpRequest, id: web::Path<String>) -> impl Responder {
    let (_, response) = when_user::<OutPlaylistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .playlist_repo()
            .find_by_id(id.into_inner().as_str())
            .await
            .map(OutPlaylistEntityDto::from),
    )
}

#[get("/playlists/default")]
async fn get_default(req: HttpRequest) -> impl Responder {
    let (_, response) = when_user::<OutPlaylistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .playlist_repo()
            .get_default_playlist()
            .await
            .map(OutPlaylistEntityDto::from),
    )
}

#[post("/playlists")]
async fn create(req: HttpRequest, payload: web::Json<InPlaylistEntityDto>) -> impl Responder {
    let (_, response) = when_admin::<OutPlaylistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .playlist_repo()
            .create(payload.0)
            .await
            .map(OutPlaylistEntityDto::from),
    )
}

#[put("playlists/{id}")]
async fn update(
    req: HttpRequest,
    id: web::Path<String>,
    payload: web::Json<InPlaylistEntityDto>,
) -> impl Responder {
    let (_, response) = when_admin::<OutPlaylistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .playlist_repo()
            .update(id.into_inner().as_str(), payload.0)
            .await
            .map(OutPlaylistEntityDto::from),
    )
}

#[delete("playlists/{id}")]
async fn delete(req: HttpRequest, id: web::Path<String>) -> impl Responder {
    let (_, response) = when_admin::<OutPlaylistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .playlist_repo()
            .delete(&id.into_inner())
            .await
            .map(OutPlaylistEntityDto::from),
    )
}
