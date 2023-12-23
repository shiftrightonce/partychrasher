use std::sync::Arc;

use orsomafo::EventDispatcherBuilder;

use crate::{
    db::DbManager,
    entity::{
        playlist::{
            playlist_event::{PlaylistAddedEvent, PlaylistDeletedEvent, PlaylistUpdatedEvent},
            PlaylistEntity,
        },
        search::SearchRepo,
    },
};

pub(crate) fn register(builder: EventDispatcherBuilder) -> EventDispatcherBuilder {
    builder
        .listen_with::<PlaylistAddedEvent>(HandleAdded)
        .listen_with::<PlaylistUpdatedEvent>(HandleUpdated)
        .listen_with::<PlaylistDeletedEvent>(HandleDeleted)
}

struct PlaylistEntry;
impl PlaylistEntry {
    async fn playlist_and_repo(&self, playlist_id: &str) -> Option<(PlaylistEntity, SearchRepo)> {
        if let Some(db_manager) = busybody::helpers::get_type::<Arc<DbManager>>() {
            if let Some(playlist) = db_manager.playlist_repo().find_by_id(playlist_id).await {
                return Some((playlist, db_manager.search_repo()));
            }
        }
        None
    }

    async fn create(&self, playlist_id: &str) {
        if let Some((playlist, repo)) = self.playlist_and_repo(playlist_id).await {
            repo.create(playlist.into()).await;
        }
    }

    async fn update(&self, playlist_id: &str) {
        if let Some((playlist, repo)) = self.playlist_and_repo(playlist_id).await {
            repo.update(playlist.into()).await;
        }
    }

    async fn delete(&self, playlist_id: &str) {
        if let Some((playlist, repo)) = self.playlist_and_repo(playlist_id).await {
            repo.delete(playlist.into()).await;
        }
    }
}

struct HandleAdded;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleAdded {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<PlaylistAddedEvent>() {
            PlaylistEntry.create(&event.playlist_id).await;
        }
    }
}

struct HandleUpdated;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleUpdated {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<PlaylistUpdatedEvent>() {
            PlaylistEntry.update(&event.playlist_id).await;
        }
    }
}

struct HandleDeleted;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleDeleted {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<PlaylistDeletedEvent>() {
            PlaylistEntry.delete(&event.playlist_id).await;
        }
    }
}
