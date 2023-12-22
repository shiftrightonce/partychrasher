use actix::Message;

#[derive(Debug, serde::Serialize, Message)]
#[rtype(result = "()")]
pub(crate) enum WebsocketMessage {
    #[serde(rename(serialize = "player_event"))]
    PlayerEvent { event: PlayerEvent },
    #[serde(rename(serialize = "playlist_event"))]
    PlaylistEvent { event: PlaylistEvent },
}

#[derive(Debug, serde::Serialize)]
pub(crate) enum PlayerEvent {
    #[serde(rename(serialize = "progress"))]
    Progress {
        position: (u64, u64, f64),
        total: (u64, u64, f64),
    },
    #[serde(rename(serialize = "play_track"))]
    PlayTrack { track_id: String },
    #[serde(rename(serialize = "play_playlist"))]
    PlayPlaylist { playlist_id: String },
    #[serde(rename(serialize = "play_album"))]
    PlayAlbum { album_id: String },
    #[serde(rename(serialize = "play"))]
    Play { play: bool },
    #[serde(rename(serialize = "skip"))]
    Skip { next: bool },
}

#[derive(Debug, serde::Serialize)]
pub(crate) enum PlaylistEvent {
    #[serde(rename(serialize = "track_added"))]
    TrackAdded {
        order_number: i64,
        playlist_id: String,
        track_id: String,
    },
    #[serde(rename(serialize = "track_removed"))]
    TrackRemoved {
        order_number: i64,
        playlist_id: String,
        track_id: String,
    },
    #[serde(rename(serialize = "default_playlist"))]
    DefaultPlaylist { playlist_id: String },
}

impl ToString for WebsocketMessage {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
