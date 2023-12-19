use crate::{db::DbConnection, entity::FromSqliteRow};

use super::{AlbumTrackEntity, InAlbumTrackEntityDto};

pub(crate) struct AlbumTrackRepo {
    pool: DbConnection,
}

impl AlbumTrackRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) {
        let sql = r#"
CREATE TABLE "album_tracks" (
    "album_id"	TEXT NOT NULL,
    "track_id"	TEXT NOT NULL,
    "metadata" TEXT,
    UNIQUE("album_id","track_id")
);
       "#;
        _ = sqlx::query(sql).execute(self.pool()).await;
    }

    pub(crate) async fn create(&self, entity: InAlbumTrackEntityDto) -> Option<AlbumTrackEntity> {
        let sql =
            "INSERT OR IGNORE INTO album_tracks (album_id, track_id, metadata) values (?, ?, ?)";
        if let Err(e) = sqlx::query(sql)
            .bind(&entity.album_id)
            .bind(&entity.track_id)
            .bind(&entity.metadata)
            .execute(self.pool())
            .await
        {
            println!("album track error: {:?}", e.to_string())
        } else {
            return self.find(&entity.album_id, &entity.track_id).await;
        }

        None
    }

    pub(crate) async fn find(&self, album_id: &str, track_id: &str) -> Option<AlbumTrackEntity> {
        let sql = "SELECT * FROM album_tracks WHERE album_id = ? AND artist_id = ?";

        if let Ok(row) = sqlx::query(sql)
            .bind(album_id)
            .bind(track_id)
            .map(AlbumTrackEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            return row;
        }

        None
    }
}
