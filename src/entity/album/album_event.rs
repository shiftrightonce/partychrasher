/// Dispatch when a album is created
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct AlbumAddedEvent {
    pub(crate) album_id: String,
}

impl orsomafo::Dispatchable for AlbumAddedEvent {}

/// Dispatched when a album is updated
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct AlbumUpdatedEvent {
    pub(crate) album_id: String,
}
impl orsomafo::Dispatchable for AlbumUpdatedEvent {}

/// Dispatched when a album is deleted
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct AlbumDeletedEvent {
    pub(crate) album_id: String,
}
impl orsomafo::Dispatchable for AlbumDeletedEvent {}
