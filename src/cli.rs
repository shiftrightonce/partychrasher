use std::io::Write;

use crate::player::PlayerCommand;

pub(crate) fn handle_request(sender: std::sync::mpsc::Sender<PlayerCommand>) {
    loop {
        let mut line = String::new();
        print!("Enter command > ");
        _ = std::io::stdout().flush();
        match std::io::stdin().read_line(&mut line) {
            Ok(_) => {
                if !line.is_empty() {
                    let pieces = line
                        .trim()
                        .split(' ')
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();

                    match pieces.first().as_ref().unwrap().as_str() {
                        "play" => {
                            if pieces.len() > 1 {
                                println!("CLI: sending play command");
                                let path = pieces[1..].join(" ");
                                _ = sender.send(PlayerCommand::Play(path));
                            }
                        }
                        "pause" => {
                            println!("CLI: sending pause command");
                            _ = sender.send(PlayerCommand::Pause)
                        }
                        "resume" => {
                            println!("CLI: sending resume command");
                            _ = sender.send(PlayerCommand::Resume)
                        }
                        "exit" => panic!("exit..."),
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
}
