use std::sync::Arc;

use actix_web::{delete, get, put, web, HttpRequest, HttpResponse, Responder, Scope};

use crate::{
    db::{DbManager, PaginatedResult, Paginator},
    entity::track::{InTrackEntityDto, OutTrackEntityDto},
    web_app::{api_response::ApiResponse, when_admin, when_user},
};

pub(crate) fn register_routes(scope: Scope) -> Scope {
    scope
        .service(get_tracks)
        // .service(create_track)
        .service(update_track)
        .service(delete_tracks)
        .service(get_a_track)
        .service(get_tracks_by_album)
        .service(get_tracks_by_playlist)
}

#[get("/tracks")]
async fn get_tracks(req: HttpRequest) -> impl Responder {
    let (_, response) = when_user::<OutTrackEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let mut paginator = Paginator::try_from(&req).unwrap();
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    let results = db_manager
        .track_repo()
        .paginate(&mut paginator)
        .await
        .into_iter()
        .map(OutTrackEntityDto::from)
        .collect::<Vec<OutTrackEntityDto>>();

    let page = PaginatedResult::<Vec<OutTrackEntityDto>>::new(results, &paginator);
    HttpResponse::Ok().json(page)
}

#[get("tracks/{id}")]
async fn get_a_track(id: web::Path<String>, req: HttpRequest) -> impl Responder {
    let (_, response) = when_user::<OutTrackEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();
    ApiResponse::into_response(
        db_manager
            .track_repo()
            .find_by_id(&id.into_inner())
            .await
            .map(OutTrackEntityDto::from),
    )
}

#[get("tracks/album/{album_id}")]
async fn get_tracks_by_album(album_id: web::Path<String>, req: HttpRequest) -> impl Responder {
    let (_, response) = when_user::<OutTrackEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();
    ApiResponse::success_response(
        db_manager
            .track_repo()
            .find_by_album_id(&album_id.into_inner())
            .await
            .into_iter()
            .map(OutTrackEntityDto::from)
            .collect::<Vec<OutTrackEntityDto>>(),
    )
}

#[get("tracks/playlist/{playlist_id}")]
async fn get_tracks_by_playlist(
    playlist_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let (_, response) = when_user::<OutTrackEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();
    ApiResponse::success_response(
        db_manager
            .track_repo()
            .find_by_playlist_id(&&playlist_id.into_inner())
            .await
            .into_iter()
            .map(OutTrackEntityDto::from)
            .collect::<Vec<OutTrackEntityDto>>(),
    )
}

// #[post("/tracks")]
// async fn create_track(req: HttpRequest) -> impl Responder {
//     "creating track"
// }

#[put("/tracks/{id}")]
async fn update_track(
    req: HttpRequest,
    id: web::Path<String>,
    payload: web::Json<InTrackEntityDto>,
) -> impl Responder {
    let (_, response) = when_admin::<OutTrackEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    ApiResponse::into_response(
        db_manager
            .track_repo()
            .update(&id.into_inner(), payload.0)
            .await
            .map(OutTrackEntityDto::from),
    )
}

#[delete("/tracks/{id}")]
async fn delete_tracks(req: HttpRequest, id: web::Path<String>) -> impl Responder {
    let (_, response) = when_admin::<OutTrackEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();
    ApiResponse::into_response(
        db_manager
            .track_repo()
            .delete(&id.into_inner())
            .await
            .map(OutTrackEntityDto::from),
    )
}
