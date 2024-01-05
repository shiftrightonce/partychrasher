#![allow(dead_code)]

use std::io::Write;

use crate::player::PlayerCommand;
const LOG_TARGET: &str = "cli";

pub(crate) fn handle_request(sender: std::sync::mpsc::Sender<PlayerCommand>) {
    loop {
        let mut line = String::new();
        print!("Enter command > ");
        _ = std::io::stdout().flush();
        if std::io::stdin().read_line(&mut line).is_ok() && !line.is_empty() {
            let pieces = line
                .trim()
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            match pieces.first().as_ref().unwrap().as_str() {
                "play" => {
                    if pieces.len() > 1 {
                        log::debug!(target: LOG_TARGET,"sending play command");
                        let path = pieces[1..].join(" ");
                        _ = sender.send(PlayerCommand::Play(path));
                    }
                }
                "pause" => {
                    log::debug!(target: LOG_TARGET,"sending pause command");
                    _ = sender.send(PlayerCommand::Pause)
                }
                "resume" => {
                    log::debug!(target: LOG_TARGET,"sending resume command");
                    _ = sender.send(PlayerCommand::Resume)
                }
                "exit" => std::process::exit(0),
                _ => (),
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        _ = std::io::stdout().flush();
    }
}
