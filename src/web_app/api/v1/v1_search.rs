use std::{collections::HashMap, sync::Arc};

use actix_web::{get, web::Query, HttpRequest, Responder, Scope};

use crate::{
    db::DbManager,
    entity::search::OutSearchHitEntityDto,
    web_app::{api_response::ApiResponse, when_user},
};

pub(crate) fn register_routes(scope: Scope) -> Scope {
    scope.service(search)
}

#[get("/search")]
async fn search(req: HttpRequest) -> impl Responder {
    let (_, response) = when_user::<OutSearchHitEntityDto>(&req).await;

    if let Some(resp) = response {
        return resp;
    }

    let db_manager = req.app_data::<Arc<DbManager>>().unwrap();
    let mut keyword = String::new();

    let query = Query::<HashMap<String, String>>::from_query(req.query_string()).unwrap();
    if let Some(query) = query.get("_q") {
        keyword = query.clone();
    }

    ApiResponse::success_response(
        db_manager
            .search_repo()
            .search(&keyword)
            .await
            .into_iter()
            .map(OutSearchHitEntityDto::from)
            .collect::<Vec<OutSearchHitEntityDto>>(),
    )
}
