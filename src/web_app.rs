use actix::*;
use futures::executor::block_on;
use futures_util::{future::FutureExt, Future};
use std::{
    pin::Pin,
    sync::{atomic::AtomicUsize, Arc},
};

use crate::{
    config::Config,
    db::DbManager,
    entity::client::ClientEntity,
    player::{PlayerCommand, PlayerUpdate},
    queue_manager::QueueManagerCommand,
    websocket::{self, server},
};
use actix_web::{
    dev::Service as _,
    web::{self, Data},
    App, HttpMessage, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer,
};

use self::{
    admin::handle_admin_command,
    api_response::ApiResponse,
    docs::dev_docs_index_handler,
    query::{
        handle_get_playlist_query, handle_next_query, handle_previous_query,
        handle_track_info_query, handle_track_search,
    },
    user::handle_user_command,
};

mod admin;
mod api_response;
mod auth_middleware;
mod docs;
mod file_server;
mod manage_client;
mod query;
mod user;

pub(crate) async fn start_webapp(
    config: &Config,
    player_sender: std::sync::mpsc::Sender<PlayerCommand>,
    queue_manager_sender: std::sync::mpsc::Sender<QueueManagerCommand>,
    mut b_receiver: tokio::sync::mpsc::UnboundedReceiver<PlayerUpdate>,
    db_manager: Arc<DbManager>,
) {
    let app_state = Arc::new(AtomicUsize::new(0));

    // start chat server actor
    let server = server::ChatServer::new(app_state.clone()).start();
    let server_copy = server.clone();

    std::thread::spawn(move || loop {
        if let Ok(msg) = b_receiver.try_recv() {
            server_copy.do_send(msg)
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    });

    let the_app_config = Data::new(config.clone());

    _ = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(player_sender.clone()))
            .app_data(Data::new(queue_manager_sender.clone()))
            .app_data(Data::new(server.clone()))
            .app_data(the_app_config.clone())
            .app_data(db_manager.clone())
            .service(actix_files::Files::new("/assets", "./static"))
            // RESTFUL API version 1
            .service(
                web::scope("/api/v1")
                    .wrap(auth_middleware::Auth)
                    .route("/user-command", web::post().to(handle_user_command))
                    .route("/admin-command", web::post().to(handle_admin_command))
                    .route("/query/next", web::get().to(handle_next_query))
                    .route("/query/previous", web::get().to(handle_previous_query))
                    .route("/query/playlist", web::get().to(handle_get_playlist_query))
                    .route(
                        "/query/info/{track}",
                        web::get().to(handle_track_info_query),
                    )
                    .route(
                        "/query/search/{keyword}",
                        web::get().to(handle_track_search),
                    )
                    .route("/stream/{id}", web::get().to(file_server::serve))
                    .route("/manage/clients", web::get().to(manage_client::get_clients))
                    .route(
                        "/manage/client/{id}",
                        web::get().to(manage_client::get_a_client),
                    )
                    .route(
                        "/manage/client",
                        web::post().to(manage_client::create_client),
                    )
                    .route(
                        "/manage/client/{id}",
                        web::put().to(manage_client::update_client),
                    )
                    .route(
                        "/manage/client/{id}",
                        web::delete().to(manage_client::delete_client),
                    )
                    .route(
                        "/manage/client/reset/{id}",
                        web::get().to(manage_client::reset_token),
                    ),
            )
            .route("/play", web::post().to(play_track))
            .route("/cmd", web::post().to(command))
            .route("/ws", web::get().to(websocket::handle_websocket))
            .route("/dev-docs", web::get().to(dev_docs_index_handler))
    })
    .bind(("0.0.0.0", 8080))
    .expect("could not bind to port: 8080")
    .run()
    .await;
}

pub(crate) async fn when_admin<R: serde::Serialize>(
    req: &HttpRequest,
) -> (bool, Option<HttpResponse>) {
    if let Ok(client) = ClientEntity::try_from(req) {
        if client.is_admin() {
            return (true, None);
        }
    }

    let response = Some(HttpResponse::Forbidden().json(ApiResponse::<R>::error(
        "Client does not have the admin role",
    )));
    (false, response)
}

pub(crate) async fn when_user<R: serde::Serialize>(
    req: &HttpRequest,
) -> (bool, Option<HttpResponse>) {
    if let Ok(client) = ClientEntity::try_from(req) {
        if !client.is_user() {
            return (true, None);
        }
    }
    let response = Some(
        HttpResponse::Forbidden().json(ApiResponse::<R>::error("Client is not authenticated")),
    );
    (false, response)
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
    match &payload.cmd {
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
    }
}
