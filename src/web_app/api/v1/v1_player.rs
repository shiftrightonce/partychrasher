use actix_web::{
    post,
    web::{self},
    HttpRequest, Responder, Scope,
};

use crate::web_app::when_admin;

pub(crate) fn register_routes(scope: Scope) -> Scope {
    scope
        .service(play_track)
        .service(play_album)
        .service(play_playlist)
        .service(control_play)
        .service(control_repeat)
        .service(control_skip)
        .service(control_volume)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
enum PlayerLocation {
    #[serde(rename(deserialize = "server", serialize = "server"))]
    Server,
    #[serde(rename(deserialize = "client", serialize = "client"))]
    Client,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct PlayTrack {
    track_id: String,
    location: PlayerLocation,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct PlayPlaylist {
    playlist_id: String,
    location: PlayerLocation,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct PlayAlbum {
    album_id: String,
    location: PlayerLocation,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Skip {
    location: PlayerLocation,
    to_track_id: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Play {
    play: bool,
    location: PlayerLocation,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Volume {
    increase: bool,
    location: PlayerLocation,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Repeat {
    one: bool,
    all: bool,
    location: PlayerLocation,
}

#[post("/player/play-track")]
async fn play_track(req: HttpRequest, payload: web::Json<PlayTrack>) -> impl Responder {
    // let (_, response) = when_admin::<String>(&req).await;

    // if let Some(r) = response {
    //     return r;
    // }

    format!("play track {:?}", payload)
}

#[post("/player/play-album")]
async fn play_album(req: HttpRequest, payload: web::Json<PlayAlbum>) -> impl Responder {
    format!("play album {:?}", payload)
}

#[post("/player/play-playlist")]
async fn play_playlist(req: HttpRequest, payload: web::Json<PlayPlaylist>) -> impl Responder {
    format!("play playlist {:?}", payload)
}

#[post("/player/control-skip")]
async fn control_skip(req: HttpRequest, payload: web::Json<Skip>) -> impl Responder {
    format!("skip {:?}", payload)
}

#[post("/player/control-play")]
async fn control_play(req: HttpRequest, payload: web::Json<Play>) -> impl Responder {
    format!("control play {:?}", payload)
}

#[post("/player/control-volume")]
async fn control_volume(req: HttpRequest, payload: web::Json<Volume>) -> impl Responder {
    format!("control volume {:?}", payload)
}

#[post("/player/control-repeat")]
async fn control_repeat(req: HttpRequest, payload: web::Json<Repeat>) -> impl Responder {
    format!("control repeat {:?}", payload)
}
