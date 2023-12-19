use actix_web::web;

use crate::web_app::auth_middleware;

mod v1_album;
mod v1_artist;
mod v1_client;
mod v1_file_server;
mod v1_playlist;
mod v1_search;
mod v1_track;

pub(crate) fn config_api_service(config: &mut web::ServiceConfig) {
    let mut api_routes = web::scope("/api/v1");

    // client routes
    api_routes = v1_client::register_routes(api_routes);
    // track routes
    api_routes = v1_track::register_routes(api_routes);
    // album routes
    api_routes = v1_album::register_routes(api_routes);
    // artist routes
    api_routes = v1_artist::register_routes(api_routes);
    // playlist routes
    api_routes = v1_playlist::register_routes(api_routes);
    // file stream routes
    api_routes = v1_file_server::register_routes(api_routes);
    // Search route
    api_routes = v1_search::register_routes(api_routes);

    config.service(api_routes.wrap(auth_middleware::Auth));
}
