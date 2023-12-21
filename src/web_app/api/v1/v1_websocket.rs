use actix::Addr;
use actix_web::{get, web, Error, HttpRequest, HttpResponse, Scope};

use crate::{
    config::Config,
    web_app::when_user,
    websocket::{self, server},
};

pub(crate) fn register_routes(scope: Scope) -> Scope {
    scope.service(websocket_handler)
}

#[get("/live/ws")]
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
    config: web::Data<Config>,
) -> Result<HttpResponse, Error> {
    let (allow, _) = when_user::<String>(&req).await;

    if !allow {
        return Ok(HttpResponse::Forbidden().body("websocket not enabled"));
    }

    websocket::handle_websocket(req, stream, srv, config).await
}
