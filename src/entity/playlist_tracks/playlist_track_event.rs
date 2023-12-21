#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct PlaylistTrackAddedEvent {
    pub(crate) track_id: String,
    pub(crate) playlist_id: String,
}

#[orsomafo::async_trait]
impl orsomafo::Dispatchable for PlaylistTrackAddedEvent {}

impl PlaylistTrackAddedEvent {
    pub(crate) fn new(playlist_id: &str, track_id: &str) -> Self {
        Self {
            track_id: track_id.to_string(),
            playlist_id: playlist_id.to_string(),
        }
    }
}
