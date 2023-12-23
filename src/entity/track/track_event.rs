/// Dispatch when a track is created
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct TrackAddedEvent {
    pub(crate) track_id: String,
}

impl orsomafo::Dispatchable for TrackAddedEvent {}

/// Dispatched when a track is updated
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct TrackUpdatedEvent {
    pub(crate) track_id: String,
}
impl orsomafo::Dispatchable for TrackUpdatedEvent {}

/// Dispatched when a track is deleted
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct TrackDeletedEvent {
    pub(crate) track_id: String,
}
impl orsomafo::Dispatchable for TrackDeletedEvent {}
