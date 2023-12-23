#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct PlaylistAddedEvent {
    pub(crate) playlist_id: String,
}

impl orsomafo::Dispatchable for PlaylistAddedEvent {}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct PlaylistUpdatedEvent {
    pub(crate) playlist_id: String,
}

impl orsomafo::Dispatchable for PlaylistUpdatedEvent {}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct PlaylistDeletedEvent {
    pub(crate) playlist_id: String,
}

impl orsomafo::Dispatchable for PlaylistDeletedEvent {}
