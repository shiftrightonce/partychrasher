use clap::{Parser, Subcommand};
use config::ConfigBuilder;
use db::setup_db_connection;
use thread_channels::setup_threads;

mod cli;
mod config;
mod db;
mod entity;
mod helper;
mod output;
mod player;
mod queue_manager;
mod seeder;
mod thread_channels;
mod web_app;
mod websocket;

#[cfg(not(target_os = "linux"))]
mod resampler;

#[actix_web::main]
async fn main() {
    pretty_env_logger::init();
    let mut config_builder = ConfigBuilder::new();
    let cli = Cli::parse();
    let mut seeding = false;
    let mut seed_total = 0;

    match cli.command {
        Some(cmd) => match cmd {
            Commands::Both => {
                config_builder = config_builder.enable_cli(true);
                config_builder = config_builder.enable_web(true);
            }
            Commands::Seed { total } => {
                seeding = true;
                seed_total = total;
                config_builder = config_builder.enable_cli(false);
                config_builder = config_builder.enable_web(false);
                config_builder = config_builder.enable_ws(false);
            }
            Commands::Cli => {
                config_builder = config_builder.enable_cli(true);
                config_builder = config_builder.enable_web(false);
                config_builder = config_builder.enable_ws(false);
            }
            Commands::Web => {
                config_builder = config_builder.enable_cli(false);
                config_builder = config_builder.enable_web(true);
                config_builder = config_builder.enable_ws(true);
            }
        },
        None => {
            config_builder = config_builder.enable_cli(false);
            config_builder = config_builder.enable_web(true);
            config_builder = config_builder.enable_ws(true);
        }
    }

    let app_config = config_builder.build();

    let db_manager = setup_db_connection(&app_config).await;

    // Setup database
    db_manager.setup_db().await;

    // // Setup all the OS threads and mpsc channels
    if app_config.is_cli_enabled() || app_config.is_web_enabled() {
        let (ws_rx, cmd_tx, queue_manager_tx) = setup_threads(&app_config);

        if app_config.is_web_enabled() {
            // Web application
            web_app::start_webapp(
                &app_config,
                cmd_tx.clone(),
                queue_manager_tx.clone(),
                ws_rx,
                db_manager,
            )
            .await;
        }
    } else if seeding {
        seeder::run_seeders(&db_manager, seed_total).await;
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Seed {
        #[arg(short, long)]
        total: u64,
    },
    Cli,
    Both,
    Web,
}
