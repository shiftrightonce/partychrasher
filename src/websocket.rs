use std::time::Instant;

use actix::Addr;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::config::Config;

pub(crate) mod server;
pub(crate) mod session;

pub(crate) async fn handle_websocket(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
    config: web::Data<Config>,
) -> Result<HttpResponse, Error> {
    if !config.is_ws_enabled() {
        return Ok(HttpResponse::NotAcceptable().body("websocket not enabled"));
    }

    ws::start(
        session::WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "main".to_owned(),
            name: None,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}
