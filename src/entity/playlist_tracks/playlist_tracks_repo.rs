use crate::{db::DbConnection, entity::FromSqliteRow};

use super::{InPlaylistTrackEntityDto, PlaylistTrackEntity};

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
    "track_id"	TEXT NOT NULL,
    "playlist_id"	TEXT NOT NULL,
    "metadata" TEXT,
    UNIQUE("track_id","playlist_id")
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
            return self
                .find(entity.playlist_id.as_str(), entity.track_id.as_str())
                .await;
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

        if existing.is_some() {
            _ = sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
                .bind(&entity.playlist_id)
                .bind(&entity.track_id)
                .execute(self.pool())
                .await;
        }

        existing
    }
}
