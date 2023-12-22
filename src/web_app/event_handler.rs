use actix::Addr;

use crate::{
    entity::playlist_tracks::{
        PlaylistIsDefaultEvent, PlaylistTrackAddedEvent, PlaylistTrackRemovedEvent,
    },
    websocket::{
        server::ChatServer,
        websocket_message::{PlaylistEvent, WebsocketMessage},
    },
};

#[derive(Debug)]
pub(crate) struct PlaylistTrackAddedHandler {
    ws_server: Addr<ChatServer>,
}

impl PlaylistTrackAddedHandler {
    pub(crate) fn new(ws_server: Addr<ChatServer>) -> Self {
        Self { ws_server }
    }
}

#[orsomafo::async_trait]
impl orsomafo::EventHandler for PlaylistTrackAddedHandler {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<PlaylistTrackAddedEvent>() {
            self.ws_server.do_send(WebsocketMessage::from(event));
        }
    }
}

impl From<PlaylistTrackAddedEvent> for WebsocketMessage {
    fn from(value: PlaylistTrackAddedEvent) -> Self {
        Self::PlaylistEvent {
            event: PlaylistEvent::TrackAdded {
                order_number: value.order_number,
                playlist_id: value.playlist_id,
                track_id: value.track_id,
            },
        }
    }
}

// ---

#[derive(Debug)]
pub(crate) struct PlaylistTrackRemovedHandler {
    ws_server: Addr<ChatServer>,
}

impl PlaylistTrackRemovedHandler {
    pub(crate) fn new(ws_server: Addr<ChatServer>) -> Self {
        Self { ws_server }
    }
}

#[orsomafo::async_trait]
impl orsomafo::EventHandler for PlaylistTrackRemovedHandler {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<PlaylistTrackRemovedEvent>() {
            self.ws_server.do_send(WebsocketMessage::from(event));
        }
    }
}

impl From<PlaylistTrackRemovedEvent> for WebsocketMessage {
    fn from(value: PlaylistTrackRemovedEvent) -> Self {
        Self::PlaylistEvent {
            event: PlaylistEvent::TrackRemoved {
                order_number: value.order_number,
                playlist_id: value.playlist_id,
                track_id: value.track_id,
            },
        }
    }
}

// -

#[derive(Debug)]
pub(crate) struct PlaylistIsDefaultEventHandler {
    ws_server: Addr<ChatServer>,
}

impl PlaylistIsDefaultEventHandler {
    pub(crate) fn new(ws_server: Addr<ChatServer>) -> Self {
        Self { ws_server }
    }
}

#[orsomafo::async_trait]
impl orsomafo::EventHandler for PlaylistIsDefaultEventHandler {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<PlaylistIsDefaultEvent>() {
            self.ws_server.do_send(WebsocketMessage::from(event));
        }
    }
}

impl From<PlaylistIsDefaultEvent> for WebsocketMessage {
    fn from(value: PlaylistIsDefaultEvent) -> Self {
        Self::PlaylistEvent {
            event: PlaylistEvent::DefaultPlaylist {
                playlist_id: value.playlist_id,
            },
        }
    }
}
