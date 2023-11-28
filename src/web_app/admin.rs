pub(crate) async fn handle_admin_command() -> impl actix_web::Responder {
    "handle admin commands"
}

#[derive(Debug, serde::Deserialize)]
enum Command {
    Player(AdminPlayerCommand),
    Manage(AdminManageCommand),
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

#[derive(Debug, serde::Deserialize)]
pub(crate) enum AdminManageCommand {
    #[serde(rename(deserialize = "add_client"))]
    AddClient(String), // Add a new client device
    #[serde(rename(deserialize = "remove_client"))]
    RemoveClient(String), // Remove a client device
    #[serde(rename(deserialize = "reset_client"))]
    Reset(String), // Resets a client token
}
