use crate::{db::DbConnection, entity::FromSqliteRow};

use super::{AlbumArtistEntity, InAlbumArtistEntityDto};

pub(crate) struct AlbumArtistRepo {
    pool: DbConnection,
}

impl AlbumArtistRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) {
        let sql = r#"
CREATE TABLE "album_artists" (
    "album_id"	TEXT NOT NULL,
    "artist_id"	TEXT NOT NULL,
    "metadata" TEXT,
    UNIQUE("album_id","artist_id")
);
       "#;
        _ = sqlx::query(sql).execute(self.pool()).await;
    }

    pub(crate) async fn create(&self, entity: InAlbumArtistEntityDto) -> Option<AlbumArtistEntity> {
        let sql = "INSERT INTO album_artists (album_id, artist_id) values (?, ?)";
        if let Err(e) = sqlx::query(sql)
            .bind(&entity.album_id)
            .bind(&entity.artist_id)
            .execute(self.pool())
            .await
        {
            println!("album artist error: {:?}", e.to_string())
        } else {
            return self.find(&entity.album_id, &entity.artist_id).await;
        }

        None
    }

    pub(crate) async fn find(&self, album_id: &str, artist_id: &str) -> Option<AlbumArtistEntity> {
        let sql = "SELECT * FROM album_artists WHERE album_id = ? AND artist_id = ?";

        if let Ok(row) = sqlx::query(sql)
            .bind(album_id)
            .bind(artist_id)
            .map(AlbumArtistEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            return row;
        }

        None
    }
}
