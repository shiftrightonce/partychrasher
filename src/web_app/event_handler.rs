use actix::Addr;

use crate::{
    entity::playlist_tracks::PlaylistTrackAddedEvent,
    websocket::{server::ChatServer, websocket_message::WebsocketMessage},
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
        Self::TrackAddedToPlaylist {
            playlist_id: value.playlist_id,
            track_id: value.track_id,
        }
    }
}
