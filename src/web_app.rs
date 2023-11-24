use crate::{player::PlayerCommand, queue_manager::QueueManagerCommand, websocket};
use actix_web::{
    web::{self, Data},
    App, HttpServer,
};

pub(crate) async fn start_webapp(
    player_sender: std::sync::mpsc::Sender<PlayerCommand>,
    queue_manager_sender: std::sync::mpsc::Sender<QueueManagerCommand>,
) {
    _ = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(player_sender.clone()))
            .app_data(Data::new(queue_manager_sender.clone()))
            .route("/hey", web::get().to(hello_world))
            .route("/play", web::post().to(play_track))
            .route("/cmd", web::post().to(command))
            .route("/ws", web::get().to(websocket::handle_websocket))
    })
    .bind(("0.0.0.0", 8080))
    .expect("could not bind to port: 8080")
    .run()
    .await;
}

async fn hello_world() -> impl actix_web::Responder {
    "Hello world, we are ready to go"
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct PlayTrackPayload {
    path: String,
}

async fn play_track(
    sender: Data<std::sync::mpsc::Sender<PlayerCommand>>,
    payload: web::Json<PlayTrackPayload>,
) -> impl actix_web::Responder {
    let track = payload.0;
    let result = sender.send(PlayerCommand::Play(track.path.clone()));

    if result.is_err() {
        log::error!("channel broken? : {:?}", result.is_err());
    }

    format!("now playing: {:?}", track.path)
}

#[derive(Debug, serde::Deserialize)]
enum Command {
    #[serde(rename(deserialize = "play"))]
    Play,
    #[serde(rename(deserialize = "play_queue"))]
    PlayQueue,
    #[serde(rename(deserialize = "reset_queue"))]
    ResetQueue,
    #[serde(rename(deserialize = "pause"))]
    Pause,
    #[serde(rename(deserialize = "resume"))]
    Resume,
    #[serde(rename(deserialize = "queue"))]
    Queue,
    #[serde(rename(deserialize = "next"))]
    Next,
    #[serde(rename(deserialize = "previous"))]
    Previous,
}

#[derive(Debug, serde::Deserialize)]
struct CommandPayload {
    cmd: Command,
    data: String,
}
async fn command(
    sender: Data<std::sync::mpsc::Sender<PlayerCommand>>,
    queue_sender: Data<std::sync::mpsc::Sender<QueueManagerCommand>>,
    payload: web::Json<CommandPayload>,
) -> impl actix_web::Responder {
    let response = match &payload.cmd {
        Command::Play if !payload.data.is_empty() => {
            _ = sender.send(PlayerCommand::Play(payload.data.clone()));
            "handled play command"
        }
        Command::PlayQueue => {
            _ = queue_sender.send(QueueManagerCommand::Play);
            "playing queue queue"
        }
        Command::ResetQueue => {
            _ = queue_sender.send(QueueManagerCommand::Reset);
            "resetting the queue"
        }
        Command::Queue if !payload.data.is_empty() => {
            _ = queue_sender.send(QueueManagerCommand::Queue(payload.data.clone()));
            "queue track"
        }
        Command::Next => {
            _ = queue_sender.send(QueueManagerCommand::Next);
            "playing next track"
        }
        Command::Previous => {
            _ = queue_sender.send(QueueManagerCommand::Previous);
            "playing previous track"
        }
        Command::Pause => {
            _ = sender.send(PlayerCommand::Pause);
            "handled pause command"
        }
        Command::Resume => {
            _ = sender.send(PlayerCommand::Resume);
            "handled resume command"
        }
        _ => "nothing to do",
    };

    response
}
