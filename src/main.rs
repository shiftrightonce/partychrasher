use clap::{Parser, Subcommand};
use config::{Config, ConfigBuilder};
use db::setup_db_connection;
use thread_channels::setup_threads;

mod cli;
mod config;
mod db;
mod entity;
mod event_registry;
mod helper;
mod output;
mod player;
mod queue_manager;
mod scanner;
mod seeder;
mod thread_channels;
mod web_app;
mod websocket;

#[cfg(not(target_os = "linux"))]
mod resampler;

const DEFAULT_DOTENV: &str = r#"
PARTY_ADMIN_ID="admin_id"
PARTY_ADMIN_TOKEN="admin_token"
PARTY_CLIENT_ID="client_id"
PARTY_CLIENT_TOKEN="client_token"
PARTY_DEFAULT_PLAYLIST="playlist_id"
PARTY_HTTP_HOST=127.0.0.1
PARTY_HTTP_PORT=8080
PARTY_DB_LOCATION="./db"
PARTY_STATIC_LOCATION="./static"
PARTY_AUDIO_FORMAT="mp3,aac,m4a,wav,ogg,wma,webm,flac"
PARTY_VIDEO_FORMAT="mp4"
PARTY_PHOTO_FORMAT="jpg,png,gif"
"#;

#[actix_web::main]
async fn main() {
    let mut attempts = 0;
    while attempts < 5 {
        if dotenvy::dotenv().is_err() {
            if let Err(e) = tokio::fs::write("./.env", DEFAULT_DOTENV).await {
                eprintln!("could not create .env file: {}", e);
            }
            attempts += 1;
        } else {
            break;
        }
    }
    pretty_env_logger::init();

    // Register events' handlers
    event_registry::register_handlers().await;

    let mut config_builder = ConfigBuilder::new();
    let cli = Cli::parse();
    let mut seeding = false;
    let mut scanning = false;
    let mut path_to_scan = String::new();
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
            Commands::Scan { path } => {
                scanning = true;
                path_to_scan = path;
                config_builder = config_builder.enable_cli(false);
                config_builder = config_builder.enable_web(false);
                config_builder = config_builder.enable_ws(false);
            }
        },
        None => {
            config_builder = config_builder.enable_cli(false);
            config_builder = config_builder.enable_web(true);
            config_builder = config_builder.enable_ws(true);
        }
    }

    let app_config = config_builder.build();

    create_db_folder(&app_config).await;

    let db_manager = setup_db_connection(&app_config).await;

    // Setup database
    db_manager.setup_db().await;

    if app_config.is_cli_enabled() || app_config.is_web_enabled() {
        // Setup all the OS threads and mpsc channels
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
        } else if app_config.is_cli_enabled() {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    } else if seeding {
        seeder::run_seeders(&db_manager, seed_total).await;
    } else if scanning {
        scanner::scan(path_to_scan, &db_manager, &app_config).await;
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
    Scan {
        #[arg(short, long)]
        path: String,
    },
}

async fn create_db_folder(config: &Config) {
    _ = tokio::fs::create_dir_all(config.db_path()).await;
    _ = tokio::fs::create_dir_all(config.static_path()).await;
    _ = tokio::fs::create_dir_all(config.artwork_path()).await;
}
