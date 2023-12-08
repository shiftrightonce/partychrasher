use actix_web::HttpRequest;

use crate::entity::client::ClientEntity;

pub(crate) async fn handle_next_query(req: HttpRequest) -> impl actix_web::Responder {
    let client = ClientEntity::try_from(&req);
    println!("current client: {:?}", &client);
    "handle next query" // returns the next track in the queue
}

pub(crate) async fn handle_previous_query() -> impl actix_web::Responder {
    "handle previous query" // returns the previous track in the queue
}

pub(crate) async fn handle_get_playlist_query() -> impl actix_web::Responder {
    "returns the current playlist"
}

pub(crate) async fn handle_track_info_query() -> impl actix_web::Responder {
    "returns track's information"
}

pub(crate) async fn handle_track_search() -> impl actix_web::Responder {
    "returns search results"
}
