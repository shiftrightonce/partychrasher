use actix::Addr;
use orsomafo::EventDispatcherBuilder;

use crate::{
    entity::playlist_tracks::{
        PlaylistIsDefaultEvent, PlaylistTrackAddedEvent, PlaylistTrackRemovedEvent,
    },
    websocket::{
        server::ChatServer,
        websocket_message::{PlaylistEvent, WebsocketMessage},
    },
};

pub(crate) fn register(builder: EventDispatcherBuilder) -> EventDispatcherBuilder {
    builder
        .listen_with::<PlaylistTrackAddedEvent>(HandleTrackAdded)
        .listen_with::<PlaylistTrackRemovedEvent>(HandleTrackRemoved)
        .listen_with::<PlaylistIsDefaultEvent>(HandleDefaultSet)
}

#[derive(Debug)]
struct HandleTrackAdded;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleTrackAdded {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<PlaylistTrackAddedEvent>() {
            if let Some(ws_server) = busybody::helpers::get_type::<Addr<ChatServer>>() {
                ws_server.do_send(WebsocketMessage::from(event));
            }
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
struct HandleTrackRemoved;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleTrackRemoved {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<PlaylistTrackRemovedEvent>() {
            if let Some(ws_server) = busybody::helpers::get_type::<Addr<ChatServer>>() {
                ws_server.do_send(WebsocketMessage::from(event));
            }
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
struct HandleDefaultSet;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleDefaultSet {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<PlaylistIsDefaultEvent>() {
            if let Some(ws_server) = busybody::helpers::get_type::<Addr<ChatServer>>() {
                ws_server.do_send(WebsocketMessage::from(event));
            }
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
