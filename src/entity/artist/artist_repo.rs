use futures::stream::TryStreamExt;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use ulid::Ulid;

use crate::{db::DbConnection, entity::FromSqliteRow, queue_manager::setup_queue_manager};

use super::{ArtistEntity, InArtistEntityDto};

pub(crate) struct ArtistRepo {
    pool: DbConnection,
}

impl ArtistRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) {
        let sql = r#"CREATE TABLE "artists" (
	"internal_id"	INTEGER,
	"id"	TEXT NOT NULL UNIQUE,
	"name"	TEXT NOT NULL UNIQUE,
	"metadata"	TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);"#;

        _ = sqlx::query(sql).execute(self.pool()).await;
    }

    pub(crate) async fn create(&self, artist: InArtistEntityDto) -> Option<ArtistEntity> {
        let sql = "INSERT INTO artists (id, name, metadata) values (?, ?, ?)";

        let id = Ulid::new().to_string().to_lowercase();

        if sqlx::query(sql)
            .bind(&id)
            .bind(artist.name)
            .bind(artist.metadata)
            .execute(self.pool())
            .await
            .is_ok()
        {
            return self.find_by_id(&id).await;
        }

        None
    }

    pub(crate) async fn upate(&self, id: &str, artist: InArtistEntityDto) -> Option<ArtistEntity> {
        let sql = "UPDATE artists SET name = ?, metadata = ? WHERE id = ?";

        if let Some(existing) = self.find_by_id(id).await {
            if sqlx::query(sql)
                .bind(artist.name)
                .bind(artist.metadata.unwrap_or(existing.metadata))
                .execute(self.pool())
                .await
                .is_ok()
            {
                return self.find_by_id(id).await;
            }
        }

        None
    }

    pub(crate) async fn find_by_id(&self, id: &str) -> Option<ArtistEntity> {
        let sql = "SELECT * FROM artists WHERE id = ?";

        if let Ok(row) = sqlx::query(sql)
            .bind(id)
            .map(ArtistEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            return row;
        }

        None
    }

    pub(crate) async fn select_random(&self, limit: i64) -> Vec<ArtistEntity> {
        let sql = "SELECT * FROM artists ORDER BY RANDOM() LIMIT ?";
        let mut results = Vec::new();

        let mut result_stream = sqlx::query(sql)
            .bind(limit)
            .map(ArtistEntity::from_row)
            .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row);
        }

        results
    }
}