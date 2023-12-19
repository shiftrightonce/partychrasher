use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpRequest, Responder, Scope};

use crate::{
    db::{DbManager, PaginatedResult, Paginator},
    entity::album::{InAlbumEntityDto, OutAlbumEntityDto},
    web_app::{api_response::ApiResponse, when_admin, when_user},
};

pub(crate) fn register_routes(scope: Scope) -> Scope {
    scope
        .service(get_albums)
        .service(create)
        .service(update)
        .service(get_an_album)
        .service(delete)
        .service(get_albums_by_track)
        .service(get_albums_by_artist)
}

#[get("/albums")]
async fn get_albums(req: HttpRequest) -> impl Responder {
    let (_, response) = when_user::<OutAlbumEntityDto>(&req).await;

    if let Some(resp) = response {
        return resp;
    }

    let mut paginator = Paginator::try_from(&req).unwrap();
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    PaginatedResult::<Vec<OutAlbumEntityDto>>::new(
        db_manager
            .album_repo()
            .paginate(&mut paginator)
            .await
            .into_iter()
            .map(OutAlbumEntityDto::from)
            .collect::<Vec<OutAlbumEntityDto>>(),
        &paginator,
    )
    .into_response()
}

#[get("albums/{id}")]
async fn get_an_album(req: HttpRequest, id: web::Path<String>) -> impl Responder {
    let (_, response) = when_user::<OutAlbumEntityDto>(&req).await;

    if let Some(resp) = response {
        return resp;
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .album_repo()
            .find_by_id(&id.into_inner())
            .await
            .map(OutAlbumEntityDto::from),
    )
}

#[get("/albums/track/{track_id}")]
async fn get_albums_by_track(req: HttpRequest, track_id: web::Path<String>) -> impl Responder {
    let (_, response) = when_user::<OutAlbumEntityDto>(&req).await;

    if let Some(resp) = response {
        return resp;
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::success_response(
        db_manager
            .album_repo()
            .find_by_track_id(&track_id.into_inner())
            .await
            .into_iter()
            .map(OutAlbumEntityDto::from)
            .collect::<Vec<OutAlbumEntityDto>>(),
    )
}

#[get("/albums/artist/{artist_id}")]
async fn get_albums_by_artist(req: HttpRequest, artist_id: web::Path<String>) -> impl Responder {
    let (_, response) = when_user::<OutAlbumEntityDto>(&req).await;

    if let Some(resp) = response {
        return resp;
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::success_response(
        db_manager
            .album_repo()
            .find_by_artist_id(&artist_id.into_inner())
            .await
            .into_iter()
            .map(OutAlbumEntityDto::from)
            .collect::<Vec<OutAlbumEntityDto>>(),
    )
}

#[post("/albums")]
async fn create(req: HttpRequest, payload: web::Json<InAlbumEntityDto>) -> impl Responder {
    let (_, response) = when_admin::<OutAlbumEntityDto>(&req).await;

    if let Some(resp) = response {
        return resp;
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .album_repo()
            .create(payload.0)
            .await
            .map(OutAlbumEntityDto::from),
    )
}

#[put("/albums/{id}")]
async fn update(
    req: HttpRequest,
    id: web::Path<String>,
    payload: web::Json<InAlbumEntityDto>,
) -> impl Responder {
    let (_, response) = when_admin::<OutAlbumEntityDto>(&req).await;

    if let Some(resp) = response {
        return resp;
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .album_repo()
            .update(&id.into_inner(), payload.0)
            .await
            .map(OutAlbumEntityDto::from),
    )
}

#[delete("albums/{id}")]
async fn delete(req: HttpRequest, id: web::Path<String>) -> impl Responder {
    let (_, response) = when_admin::<OutAlbumEntityDto>(&req).await;

    if let Some(resp) = response {
        return resp;
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .album_repo()
            .delete(&id.into_inner())
            .await
            .map(OutAlbumEntityDto::from),
    )
}
