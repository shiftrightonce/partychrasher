use std::sync::Arc;

use orsomafo::EventDispatcherBuilder;

use crate::{
    db::DbManager,
    entity::{
        album::{AlbumAddedEvent, AlbumDeletedEvent, AlbumEntity, AlbumUpdatedEvent},
        search::SearchRepo,
    },
};

pub(crate) fn register(builder: EventDispatcherBuilder) -> EventDispatcherBuilder {
    builder
        .listen_with::<AlbumAddedEvent>(HandleAdded)
        .listen_with::<AlbumUpdatedEvent>(HandleUpdated)
        .listen_with::<AlbumDeletedEvent>(HandleDeleted)
}

struct AlbumEntry;

impl AlbumEntry {
    async fn album_and_repo(&self, album_id: &str) -> Option<(AlbumEntity, SearchRepo)> {
        if let Some(db_manager) = busybody::helpers::get_type::<Arc<DbManager>>() {
            if let Some(album) = db_manager.album_repo().find_by_id(album_id).await {
                return Some((album, db_manager.search_repo()));
            }
        }
        None
    }

    async fn create(&self, album_id: &str) {
        if let Some((album, repo)) = self.album_and_repo(album_id).await {
            repo.create(album.into()).await;
        }
    }

    async fn update(&self, album_id: &str) {
        if let Some((album, repo)) = self.album_and_repo(album_id).await {
            repo.update(album.into()).await;
        }
    }

    async fn delete(&self, album_id: &str) {
        if let Some((album, repo)) = self.album_and_repo(album_id).await {
            repo.delete(album.into()).await;
        }
    }
}

struct HandleAdded;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleAdded {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<AlbumAddedEvent>() {
            AlbumEntry.create(&event.album_id).await;
        }
    }
}

struct HandleUpdated;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleUpdated {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<AlbumUpdatedEvent>() {
            AlbumEntry.update(&event.album_id).await;
        }
    }
}

struct HandleDeleted;

#[orsomafo::async_trait]
impl orsomafo::EventHandler for HandleDeleted {
    async fn handle(&self, dispatched: &orsomafo::DispatchedEvent) {
        if let Some(event) = dispatched.the_event::<AlbumDeletedEvent>() {
            AlbumEntry.delete(&event.album_id).await;
        }
    }
}
