use actix_web::web;

use crate::web_app::auth_middleware;

mod v1_client;
mod v1_file_server;
mod v1_track;

pub(crate) fn config_api_service(config: &mut web::ServiceConfig) {
    let mut api_routes = web::scope("/api/v1");

    // client routes
    api_routes = v1_client::register_routes(api_routes);
    // track routes
    api_routes = v1_track::register_routes(api_routes);
    // file stream routes
    api_routes = v1_file_server::register_routes(api_routes);

    config.service(api_routes.wrap(auth_middleware::Auth));
    // app.service(service);
}