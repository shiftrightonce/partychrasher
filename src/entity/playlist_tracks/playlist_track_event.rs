#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct PlaylistTrackAddedEvent {
    pub(crate) order_number: i64,
    pub(crate) track_id: String,
    pub(crate) playlist_id: String,
}

#[orsomafo::async_trait]
impl orsomafo::Dispatchable for PlaylistTrackAddedEvent {}

impl PlaylistTrackAddedEvent {
    pub(crate) fn new(order_number: i64, playlist_id: &str, track_id: &str) -> Self {
        Self {
            order_number,
            track_id: track_id.to_string(),
            playlist_id: playlist_id.to_string(),
        }
    }
}

// -
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct PlaylistTrackRemovedEvent {
    pub(crate) order_number: i64,
    pub(crate) track_id: String,
    pub(crate) playlist_id: String,
}

#[orsomafo::async_trait]
impl orsomafo::Dispatchable for PlaylistTrackRemovedEvent {}

impl PlaylistTrackRemovedEvent {
    pub(crate) fn new(order_number: i64, playlist_id: &str, track_id: &str) -> Self {
        Self {
            order_number,
            track_id: track_id.to_string(),
            playlist_id: playlist_id.to_string(),
        }
    }
}

// -

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct PlaylistIsDefaultEvent {
    pub(crate) playlist_id: String,
}

#[orsomafo::async_trait]
impl orsomafo::Dispatchable for PlaylistIsDefaultEvent {}

impl PlaylistIsDefaultEvent {
    pub(crate) fn new(playlist_id: &str) -> Self {
        Self {
            playlist_id: playlist_id.to_string(),
        }
    }
}
