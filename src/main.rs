use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use player::PlayerUpdate;

use crate::player::PlayerCommand;

mod cli;
mod output;
mod player;
mod queue_manager;
mod web_app;
mod websocket;

#[cfg(not(target_os = "linux"))]
mod resampler;

#[actix_web::main]
async fn main() {
    pretty_env_logger::init();
    let (sender, receiver) = std::sync::mpsc::channel::<PlayerCommand>();
    let (sync_sender, sync_receiver) = std::sync::mpsc::channel::<PlayerUpdate>();

    // Queue Manager
    let manager = queue_manager::setup_queue_manager(sender.clone());

    // Actual player
    std::thread::spawn(move || {
        player::handle_request(receiver, sync_sender.clone());
    });

    // Cli for quick debugging
    let sender1 = sender.clone();
    std::thread::spawn(move || {
        cli::handle_request(sender1.clone());
    });

    // Web application
    // web_app::start_webapp(sender.clone(), manager.clone()).await;

    // display updates
    // std::thread::spawn(move || loop {
    //     if let Ok(update) = sync_receiver.recv() {
    //         match update {
    //             PlayerUpdate::Progress { position, total } => {
    //                 println!("{:?}/{}", position, total);
    //             }
    //         }
    //     }
    // });
}
