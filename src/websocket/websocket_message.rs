use actix::Message;

#[derive(Debug, serde::Serialize, Message)]
#[rtype(result = "()")]
pub(crate) enum WebsocketMessage {
    PlayProgress {
        position: (u64, u64, f64),
        total: (u64, u64, f64),
    },
    TrackAddedToPlaylist {
        playlist_id: String,
        track_id: String,
    },
}

impl ToString for WebsocketMessage {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
