/// Dispatch when a artist is created
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct ArtistAddedEvent {
    pub(crate) artist_id: String,
}

impl orsomafo::Dispatchable for ArtistAddedEvent {}

/// Dispatched when a artist is updated
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct ArtistUpdatedEvent {
    pub(crate) artist_id: String,
}
impl orsomafo::Dispatchable for ArtistUpdatedEvent {}
