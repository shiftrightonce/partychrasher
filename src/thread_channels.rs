use crate::{
    cli,
    config::Config,
    player::{self, PlayerCommand, PlayerUpdate},
    queue_manager::{self, QueueManagerCommand},
};

pub(crate) fn setup_threads(
    config: &Config,
) -> (
    tokio::sync::mpsc::UnboundedReceiver<PlayerUpdate>,
    std::sync::mpsc::Sender<PlayerCommand>,
    std::sync::mpsc::Sender<QueueManagerCommand>,
) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<PlayerUpdate>();
    let update_tx = setup_progress_thread(tx);
    let cmd_tx = setup_player_thread(update_tx);

    let manager_tx = queue_manager::setup_queue_manager(cmd_tx.clone());

    // TODO: First check if we should have the cli running
    if config.is_cli_enabled() {
        setup_cli_thread(cmd_tx.clone());
    }

    (rx, cmd_tx, manager_tx)
}

fn setup_progress_thread(
    websocket_tx: tokio::sync::mpsc::UnboundedSender<PlayerUpdate>,
) -> std::sync::mpsc::Sender<PlayerUpdate> {
    let (tx, rx) = std::sync::mpsc::channel::<PlayerUpdate>();

    std::thread::spawn(move || loop {
        if let Ok(update) = rx.try_recv() {
            _ = websocket_tx.send(update);
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    });

    tx
}

fn setup_player_thread(
    progress_tx: std::sync::mpsc::Sender<PlayerUpdate>,
) -> std::sync::mpsc::Sender<PlayerCommand> {
    let (sender, receiver) = std::sync::mpsc::channel::<PlayerCommand>();

    std::thread::spawn(move || {
        player::handle_request(receiver, progress_tx);
    });

    sender
}

fn setup_cli_thread(cmd_tx: std::sync::mpsc::Sender<PlayerCommand>) {
    std::thread::spawn(move || {
        cli::handle_request(cmd_tx.clone());
    });
}
