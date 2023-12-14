use std::sync::Arc;

use actix_web::{get, post, web, HttpRequest, Responder, Scope};

use crate::{
    db::{DbManager, PaginatedResult, Paginator},
    entity::artist::{InArtistEntityDto, OutArtistEntityDto},
    web_app::{api_response::ApiResponse, when_admin, when_user},
};

pub(crate) fn register_routes(scope: Scope) -> Scope {
    scope
        .service(get_artists)
        .service(get_an_artist)
        .service(create)
        .service(update)
        .service(get_artists_by_track)
        .service(get_artists_by_album)
}

#[get("artists")]
async fn get_artists(req: HttpRequest) -> impl Responder {
    let (_, response) = when_user::<OutArtistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let mut paginator = Paginator::try_from(&req).unwrap();
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    PaginatedResult::<Vec<OutArtistEntityDto>>::new(
        db_manager
            .artist_repo()
            .paginate(&mut paginator)
            .await
            .into_iter()
            .map(OutArtistEntityDto::from)
            .collect::<Vec<OutArtistEntityDto>>(),
        &paginator,
    )
    .into_response()
}

#[get("/artists/{id}")]
async fn get_an_artist(req: HttpRequest, id: web::Path<String>) -> impl Responder {
    let (_, response) = when_user::<OutArtistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .artist_repo()
            .find_by_id(&id.into_inner())
            .await
            .map(OutArtistEntityDto::from),
    )
}

#[post("/artists")]
async fn create(req: HttpRequest, payload: web::Json<InArtistEntityDto>) -> impl Responder {
    let (_, response) = when_admin::<OutArtistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .artist_repo()
            .create(payload.0)
            .await
            .map(OutArtistEntityDto::from),
    )
}

#[post("/artists/{id}")]
async fn update(
    req: HttpRequest,
    id: web::Path<String>,
    payload: web::Json<InArtistEntityDto>,
) -> impl Responder {
    let (_, response) = when_admin::<OutArtistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .artist_repo()
            .update(&id.into_inner(), payload.0)
            .await
            .map(OutArtistEntityDto::from),
    )
}

#[get("/artists/track/{track_id}")]
async fn get_artists_by_track(req: HttpRequest, track_id: web::Path<String>) -> impl Responder {
    let (_, response) = when_user::<OutArtistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();
    ApiResponse::success_response(
        db_manager
            .artist_repo()
            .find_by_track_id(&track_id.into_inner())
            .await
            .into_iter()
            .map(OutArtistEntityDto::from)
            .collect::<Vec<OutArtistEntityDto>>(),
    )
}

#[get("/artists/album/{album_id}")]
async fn get_artists_by_album(req: HttpRequest, album_id: web::Path<String>) -> impl Responder {
    let (_, response) = when_user::<OutArtistEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();
    ApiResponse::success_response(
        db_manager
            .artist_repo()
            .find_by_album_id(&album_id.into_inner())
            .await
            .into_iter()
            .map(OutArtistEntityDto::from)
            .collect::<Vec<OutArtistEntityDto>>(),
    )
}
