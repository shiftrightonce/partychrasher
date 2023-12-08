use std::sync::Arc;

use actix_web::{
    web::{self},
    HttpRequest, HttpResponse, Responder,
};

use crate::{
    db::{DbManager, PaginatedResult, Paginator},
    entity::client::{InClientEntityDto, OutApiTokenDto, OutClientEntityDto},
    web_app::{api_response::ApiResponse, when_admin},
};

pub(crate) async fn get_clients(req: HttpRequest) -> impl Responder {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;
    if response.is_some() {
        return response.unwrap();
    }
    let mut paginator = Paginator::try_from(&req).unwrap();
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    let results = db_manager
        .client_repo()
        .paginate(&mut paginator)
        .await
        .into_iter()
        .map(OutClientEntityDto::from)
        .collect::<Vec<OutClientEntityDto>>();

    let page = PaginatedResult::<Vec<OutClientEntityDto>>::new(results, &paginator);
    HttpResponse::Ok().json(page)
}

pub(crate) async fn get_a_client(id: web::Path<String>, req: HttpRequest) -> impl Responder {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;
    let select_id = id.into_inner();

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    match db_manager.client_repo().find_by_id(&select_id).await {
        Some(new_client) => HttpResponse::Ok().json(ApiResponse::<OutClientEntityDto>::success(
            OutClientEntityDto::from(new_client),
        )),
        None => HttpResponse::Forbidden().json(ApiResponse::<OutClientEntityDto>::error(
            "Could not find user with the specified ID",
        )),
    }
}

pub(crate) async fn create_client(
    req: HttpRequest,
    payload: web::Json<InClientEntityDto>,
) -> impl Responder {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();
    if payload.name.is_some()
        && db_manager
            .client_repo()
            .find_by_name(payload.name.as_ref().unwrap())
            .await
            .is_some()
    {
        return HttpResponse::Forbidden().json(ApiResponse::<OutClientEntityDto>::error(
            "A client with this name already exist",
        ));
    }

    return match db_manager.client_repo().create(payload.into_inner()).await {
        Some(new_client) => HttpResponse::Ok().json(ApiResponse::<OutClientEntityDto>::success(
            OutClientEntityDto::from(new_client),
        )),
        None => HttpResponse::Forbidden().json(ApiResponse::<OutClientEntityDto>::error(
            "Could not create new client. Please review data",
        )),
    };
}

pub(crate) async fn update_client(
    id: web::Path<String>,
    req: HttpRequest,
    payload: web::Json<InClientEntityDto>,
) -> impl Responder {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;
    let to_update_id = id.into_inner();

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();
    if payload.name.is_some() {
        if let Some(existing) = db_manager
            .client_repo()
            .find_by_name(payload.name.as_ref().unwrap())
            .await
        {
            if existing.id != to_update_id {
                return HttpResponse::Forbidden().json(ApiResponse::<OutClientEntityDto>::error(
                    "A client with this name already exist",
                ));
            }
        }
    }

    return match db_manager
        .client_repo()
        .update(&to_update_id, payload.into_inner())
        .await
    {
        Some(new_client) => HttpResponse::Ok().json(ApiResponse::<OutClientEntityDto>::success(
            OutClientEntityDto::from(new_client),
        )),
        None => HttpResponse::Forbidden().json(ApiResponse::<OutClientEntityDto>::error(
            "Could not create new client. Please review data",
        )),
    };
}

pub(crate) async fn delete_client(id: web::Path<String>, req: HttpRequest) -> impl Responder {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;
    let to_delete_id = id.into_inner();

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    match db_manager.client_repo().delete(&to_delete_id).await {
        Some(deleted_client) => HttpResponse::Ok().json(
            ApiResponse::<OutClientEntityDto>::success(OutClientEntityDto::from(deleted_client)),
        ),
        None => HttpResponse::Forbidden().json(ApiResponse::<OutClientEntityDto>::error(
            "Could not deleted this client",
        )),
    }
}

pub(crate) async fn reset_token(id: web::Path<String>, req: HttpRequest) -> impl Responder {
    let (_, response) = when_admin::<OutApiTokenDto>(&req).await;
    let to_reset_id = id.into_inner();

    if response.is_some() {
        return response.unwrap();
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    match db_manager.client_repo().reset_secret(&to_reset_id).await {
        Some(deleted_client) => HttpResponse::Ok().json(ApiResponse::<OutApiTokenDto>::success(
            OutApiTokenDto::from(deleted_client),
        )),
        None => HttpResponse::Forbidden().json(ApiResponse::<OutApiTokenDto>::error(
            "Could not reset this client's api token",
        )),
    }
}
