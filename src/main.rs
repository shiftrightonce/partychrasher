use config::ConfigBuilder;
use thread_channels::setup_threads;

mod cli;
mod config;
mod output;
mod player;
mod queue_manager;
mod thread_channels;
mod web_app;
mod websocket;

#[cfg(not(target_os = "linux"))]
mod resampler;

#[actix_web::main]
async fn main() {
    pretty_env_logger::init();

    let app_config = ConfigBuilder::new()
        .enable_cli(true) // Hardcoded  for now
        .enable_ws(true) // Hardcoded for now
        .build();

    // Setup all the os threads and mpsc channels
    let (ws_rx, cmd_tx, queue_manager_tx) = setup_threads(&app_config);

    // Web application
    web_app::start_webapp(&app_config, cmd_tx.clone(), queue_manager_tx.clone(), ws_rx).await;
}
