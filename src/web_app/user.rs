use actix_web::{
    web::{self, Data},
    HttpResponse,
};

use crate::{player::PlayerCommand, queue_manager::QueueManagerCommand};

pub(crate) async fn handle_user_command(
    _sender: Data<std::sync::mpsc::Sender<PlayerCommand>>,
    _queue_sender: Data<std::sync::mpsc::Sender<QueueManagerCommand>>,
    command: web::Json<UserCommand>,
) -> impl actix_web::Responder {
    // 1. Validate the User

    let x = command.into_inner();
    HttpResponse::Ok().json(x)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) enum UserCommand {
    #[serde(rename(deserialize = "queue"))]
    Queue(String), // Request to queue a track
}
