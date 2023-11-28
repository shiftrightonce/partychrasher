pub(crate) async fn handle_next_query() -> impl actix_web::Responder {
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
