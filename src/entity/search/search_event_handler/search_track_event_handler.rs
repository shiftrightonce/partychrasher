use std::sync::Arc;

use orsomafo::EventDispatcherBuilder;

use crate::{
    db::DbManager,
    entity::{
        search::SearchRepo,
        track::{
            track_event::{self, TrackAddedEvent, TrackDeletedEvent, TrackUpdatedEvent},
            TrackEntity,
        },
    },
};

pub(crate) fn register(builder: EventDispatcherBuilder) -> EventDispatcherBuilder {
    builder
        .listen_with::<track_event::TrackAddedEvent>(HandleAdded)
        .listen_with::<track_event::TrackUpdatedEvent>(HandleUpdated)
        .listen_with::<track_event::TrackDeletedEvent>(HandleDeleted)
}

struct TrackEntry;
impl TrackEntry {
    async fn track_and_repo(&self, track_id: &str) -> Option<(TrackEntity, SearchRepo)> {
        if let Some(db_manager) = busybody::helpers::get_type::<Arc<DbManager>>() {
            if let Some(track) = db_manager.track_repo().find_by_id(track_id).await {
                return Some((track, db_manager.search_repo()));
            }
        }

        None
    }

    async fn create(&self, track_id: &str) {
        if let Some((track, repo)) = self.track_and_repo(track_id).await {
            repo.create(track.into()).await;
        }
    }
    async fn update(&self, track_id: &str) {
        if let Some((track, repo)) = self.track_and_repo(track_id).await {
            repo.update(track.into()).await;
        }
    }

    async fn delete(&self, track_id: &str) {
        if let Some((track, repo)) = self.track_and_repo(track_id).await {
            repo.delete(track.into()).await;
        }
    }
}

struct HandleAdded;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleAdded {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<TrackAddedEvent>() {
            TrackEntry.create(&event.track_id).await;
        }
    }
}

struct HandleUpdated;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleUpdated {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<TrackUpdatedEvent>() {
            TrackEntry.update(&event.track_id).await;
        }
    }
}

struct HandleDeleted;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleDeleted {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<TrackDeletedEvent>() {
            TrackEntry.delete(&event.track_id).await;
        }
    }
}
