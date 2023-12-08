pub(crate) async fn handle_admin_command() -> impl actix_web::Responder {
    "handle admin commands"
}

#[derive(Debug, serde::Deserialize)]
pub(crate) enum AdminPlayerCommand {
    #[serde(rename(deserialize = "play"))]
    Play(String),
    #[serde(rename(deserialize = "play_queue"))]
    PlayQueue(String),
    #[serde(rename(deserialize = "reset_queue"))]
    ResetQueue(String),
    #[serde(rename(deserialize = "pause"))]
    Pause,
    #[serde(rename(deserialize = "resume"))]
    Resume,
    #[serde(rename(deserialize = "queue"))]
    Queue(String),
    #[serde(rename(deserialize = "next"))]
    Next,
    #[serde(rename(deserialize = "previous"))]
    Previous,
}
