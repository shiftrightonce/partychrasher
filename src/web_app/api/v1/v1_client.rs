use std::sync::Arc;

use actix_web::{
    cookie::Cookie,
    delete, get, post, put,
    web::{self},
    HttpRequest, HttpResponse, Responder, Scope,
};

use crate::{
    db::{DbManager, PaginatedResult, Paginator},
    entity::client::{ClientEntity, InClientEntityDto, OutApiTokenDto, OutClientEntityDto},
    web_app::{api_response::ApiResponse, when_admin, when_user},
};

pub(crate) fn register_routes(scope: Scope) -> Scope {
    scope
        .service(get_clients)
        .service(get_me)
        .service(get_a_client)
        .service(create_client)
        .service(update_client)
        .service(delete_client)
        .service(reset_token)
}

pub(crate) fn register_open_routes(scope: Scope) -> Scope {
    scope.service(authenticate)
}

#[get("/clients")]
async fn get_clients(req: HttpRequest) -> impl Responder {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;
    if let Some(resp) = response {
        return resp;
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

#[get("clients/me")]
async fn get_me(req: HttpRequest) -> impl Responder {
    let (_, response) = when_user::<ClientEntity>(&req).await;

    if let Some(resp) = response {
        return resp;
    }

    ApiResponse::into_response(ClientEntity::try_from(&req).ok())
}

#[get("/clients/{id}")]
async fn get_a_client(id: web::Path<String>, req: HttpRequest) -> impl Responder {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;
    let select_id = id.into_inner();

    if let Some(resp) = response {
        return resp;
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

#[post("/clients")]
async fn create_client(req: HttpRequest, payload: web::Json<InClientEntityDto>) -> impl Responder {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;

    if let Some(resp) = response {
        return resp;
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

    match db_manager.client_repo().create(payload.into_inner()).await {
        Some(new_client) => HttpResponse::Ok().json(ApiResponse::<OutClientEntityDto>::success(
            OutClientEntityDto::from(new_client),
        )),
        None => HttpResponse::Forbidden().json(ApiResponse::<OutClientEntityDto>::error(
            "Could not create new client. Please review data",
        )),
    }
}

#[put("/clients/{id}")]
async fn update_client(
    id: web::Path<String>,
    req: HttpRequest,
    payload: web::Json<InClientEntityDto>,
) -> impl Responder {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;
    let to_update_id = id.into_inner();

    if let Some(resp) = response {
        return resp;
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

    match db_manager
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
    }
}

#[delete("/clients/{id}")]
async fn delete_client(id: web::Path<String>, req: HttpRequest) -> impl Responder {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;
    let to_delete_id = id.into_inner();

    if let Some(resp) = response {
        return resp;
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

#[get("/clients/token-reset/{id}")]
async fn reset_token(id: web::Path<String>, req: HttpRequest) -> impl Responder {
    let (_, response) = when_admin::<OutApiTokenDto>(&req).await;
    let to_reset_id = id.into_inner();

    if let Some(resp) = response {
        return resp;
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

#[get("/clients/auth/{id}")]
pub(crate) async fn authenticate(
    req: HttpRequest,
    login_token: web::Path<String>,
) -> impl Responder {
    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();

    if let Some(client) = db_manager
        .client_repo()
        .find_by_login_token(login_token.as_str())
        .await
    {
        let mut cookie = Cookie::new("_party_t", client.api_token());
        cookie.set_same_site(Some(actix_web::cookie::SameSite::None));
        cookie.set_secure(Some(false));
        cookie.set_http_only(true);
        cookie.set_path("/");

        let mut response = ApiResponse::success_response(client);
        _ = response.add_cookie(&cookie);
        response
    } else {
        ApiResponse::<ClientEntity>::into_response(None)
    }
}
