use crate::{db::DbConnection, entity::FromSqliteRow};

use super::{ArtistTrackEntity, InArtistTrackEntityDto};

pub(crate) struct ArtistTrackRepo {
    pool: DbConnection,
}

impl ArtistTrackRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) {
        let sql = r#"
CREATE TABLE "artist_tracks" (
    "artist_id"	TEXT NOT NULL,
    "track_id"	TEXT NOT NULL,
     "is_feature" INTEGER DEFAULT 0,
    "metadata" TEXT,
    UNIQUE("artist_id","track_id")
);
       "#;
        _ = sqlx::query(sql).execute(self.pool()).await;
    }

    pub(crate) async fn create(&self, entity: InArtistTrackEntityDto) -> Option<ArtistTrackEntity> {
        let sql = "INSERT OR IGNORE INTO artist_tracks (artist_id, track_id, is_feature, metadata) values (?, ?, ? ,?)";
        if let Err(e) = sqlx::query(sql)
            .bind(&entity.artist_id)
            .bind(&entity.track_id)
            .bind(entity.is_feature)
            .bind(&entity.metadata.unwrap_or_default())
            .execute(self.pool())
            .await
        {
            println!("artist track error: {:?}", e.to_string())
        } else {
            return self.find(&entity.artist_id, &entity.track_id).await;
        }

        None
    }

    pub(crate) async fn find(&self, artist_id: &str, track_id: &str) -> Option<ArtistTrackEntity> {
        let sql = "SELECT * FROM artist_tracks WHERE artist_id = ? AND artist_id = ?";

        if let Ok(row) = sqlx::query(sql)
            .bind(artist_id)
            .bind(track_id)
            .map(ArtistTrackEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            return row;
        }

        None
    }
}
