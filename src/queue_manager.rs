use std::sync::{atomic::AtomicUsize, RwLock};

use crate::player::PlayerCommand;

pub(crate) enum QueueManagerCommand {
    Next,
    Previous,
    Play,
    Reset,
    Queue(String),
}

pub(crate) fn setup_queue_manager(
    sender: std::sync::mpsc::Sender<PlayerCommand>,
) -> std::sync::mpsc::Sender<QueueManagerCommand> {
    let (queue_sender, receiver) = std::sync::mpsc::channel::<QueueManagerCommand>();
    let mut manager = QueueManager::new(sender);
    std::thread::spawn(move || loop {
        if let Ok(cmd) = receiver.recv() {
            match cmd {
                QueueManagerCommand::Next => manager.next(),
                QueueManagerCommand::Previous => manager.previous(),
                QueueManagerCommand::Play => manager.play_queue(),
                QueueManagerCommand::Reset => manager.reset(),
                QueueManagerCommand::Queue(track) => {
                    let count = manager.queue(&track);
                    log::debug!("total tracks queued: {}", count)
                }
            }
        }
    });

    queue_sender
}

#[derive(Debug)]
pub(crate) struct QueueManager {
    current: AtomicUsize,
    queue: RwLock<Vec<String>>, // TODO: Fetch the queue from a persistent storage. Do not keep the queue in memory
    sender: std::sync::mpsc::Sender<PlayerCommand>,
}

impl QueueManager {
    pub(crate) fn new(sender: std::sync::mpsc::Sender<PlayerCommand>) -> Self {
        Self {
            current: AtomicUsize::default(),
            queue: RwLock::new(Vec::new()),
            sender,
        }
    }
    pub(crate) fn next(&mut self) {
        let index = self.current.load(std::sync::atomic::Ordering::Relaxed) + 1;
        self.play_by_index_and_set(index);
    }

    pub(crate) fn previous(&mut self) {
        let mut index = self.current.load(std::sync::atomic::Ordering::Relaxed);
        if index != 0 {
            index = index - 1;
        }

        self.play_by_index_and_set(index);
    }

    pub(crate) fn queue(&self, track: &str) -> usize {
        if let Ok(mut lock) = self.queue.write() {
            lock.push(track.to_string());
            return lock.len();
        }
        0
    }

    pub(crate) fn play(&self, track: &str) {
        _ = self.sender.send(PlayerCommand::Play(track.to_string()))
    }

    fn play_by_index(&self, index: usize) -> bool {
        if let Ok(lock) = self.queue.read() {
            if let Some(track) = lock.get(index) {
                self.play(track);
            }
            true
        } else {
            false
        }
    }
    fn play_by_index_and_set(&mut self, index: usize) -> bool {
        if self.play_by_index(index) {
            *self.current.get_mut() = index;
            true
        } else {
            false
        }
    }
    fn play_queue(&mut self) {
        self.play_by_index(self.current.load(std::sync::atomic::Ordering::Relaxed));
    }

    fn reset(&mut self) {
        // TODO: This will do a sql "delete * from queue ...."
        if let Ok(mut lock) = self.queue.write() {
            lock.clear();
        }
    }
}
