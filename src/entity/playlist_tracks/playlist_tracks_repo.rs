use crate::{db::DbConnection, entity::FromSqliteRow};

use super::{
    InPlaylistTrackEntityDto, PlaylistTrackAddedEvent, PlaylistTrackEntity,
    PlaylistTrackRemovedEvent,
};

pub(crate) struct PlaylistTracksRepo {
    pool: DbConnection,
}

impl PlaylistTracksRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) {
        let sql = r#"
CREATE TABLE "playlist_tracks" (
	"internal_id"	INTEGER,
    "track_id"	TEXT NOT NULL,
    "playlist_id"	TEXT NOT NULL,
    "metadata" TEXT,
    UNIQUE("track_id","playlist_id"),
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);
       "#;

        _ = sqlx::query(sql).execute(self.pool()).await;
    }

    pub(crate) async fn create(
        &self,
        entity: InPlaylistTrackEntityDto,
    ) -> Option<PlaylistTrackEntity> {
        let sql = "INSERT INTO playlist_tracks (playlist_id, track_id, metadata) values (?, ?, ?)";
        if sqlx::query(sql)
            .bind(&entity.playlist_id)
            .bind(&entity.track_id)
            .bind(&entity.metadata)
            .execute(self.pool())
            .await
            .is_ok()
        {
            let result = self
                .find(entity.playlist_id.as_str(), entity.track_id.as_str())
                .await;

            // Dispatch track added event
            orsomafo::Dispatchable::dispatch_event(PlaylistTrackAddedEvent::new(
                result.as_ref().unwrap().internal_id,
                &result.as_ref().unwrap().playlist_id,
                &result.as_ref().unwrap().track_id,
            ));

            return result;
        }

        None
    }

    pub(crate) async fn find(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Option<PlaylistTrackEntity> {
        let sql = "SELECT * FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?";

        if let Ok(row) = sqlx::query(sql)
            .bind(playlist_id)
            .bind(track_id)
            .map(PlaylistTrackEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            return row;
        }

        None
    }

    pub(crate) async fn delete(
        &self,
        entity: InPlaylistTrackEntityDto,
    ) -> Option<PlaylistTrackEntity> {
        let existing = self.find(&entity.playlist_id, &entity.track_id).await;

        if existing.is_some()
            && sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
                .bind(&entity.playlist_id)
                .bind(&entity.track_id)
                .execute(self.pool())
                .await
                .is_ok()
        {
            // Dispatch track remove event
            orsomafo::Dispatchable::dispatch_event(PlaylistTrackRemovedEvent::new(
                existing.as_ref().unwrap().internal_id,
                &existing.as_ref().unwrap().playlist_id,
                &existing.as_ref().unwrap().track_id,
            ));
        }

        existing
    }
}
