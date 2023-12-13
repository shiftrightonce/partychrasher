use futures::stream::TryStreamExt;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use ulid::Ulid;

use crate::{db::DbConnection, entity::FromSqliteRow, queue_manager::setup_queue_manager};

use super::{AlbumEntity, InAlbumEntityDto};

pub(crate) struct AlbumRepo {
    pool: DbConnection,
}

impl AlbumRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) {
        let sql = r#"CREATE TABLE "albums" (
	"internal_id"	INTEGER,
	"id"	TEXT,
	"title"	TEXT,
	"metadata"	TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);"#;

        _ = sqlx::query(sql).execute(self.pool()).await;
    }

    pub(crate) async fn create(&self, album: InAlbumEntityDto) -> Option<AlbumEntity> {
        let sql = "INSERT INTO albums (id, title, metadata) values (?, ?, ?)";

        let id = Ulid::new().to_string().to_lowercase();

        if sqlx::query(sql)
            .bind(&id)
            .bind(album.title)
            .bind(album.metadata.unwrap_or_default())
            .execute(self.pool())
            .await
            .is_ok()
        {
            return self.find_by_id(&id).await;
        }
        None
    }

    pub(crate) async fn update(&self, id: &str, album: InAlbumEntityDto) -> Option<AlbumEntity> {
        let sql = "UPDATE albums set title =?, metadata = ? WHERE id = ?";
        if let Some(existing) = self.find_by_id(id).await {
            if sqlx::query(sql)
                .bind(album.title)
                .bind(album.metadata.unwrap_or(existing.metadata))
                .bind(id)
                .execute(self.pool())
                .await
                .is_ok()
            {
                return self.find_by_id(id).await;
            }
        }

        None
    }

    pub(crate) async fn find_by_id(&self, id: &str) -> Option<AlbumEntity> {
        let sql = "SELECT * from albums WHERE id = ?";

        if let Ok(row) = sqlx::query(sql)
            .bind(id)
            .map(AlbumEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            row
        } else {
            None
        }
    }

    pub(crate) async fn search(&self, keyword: &str) -> Vec<AlbumEntity> {
        let sql = "SELECT * FROM albums WHERE title LIKE = ? LIMIT 100";
        let mut results = Vec::new();

        let mut result_stream = sqlx::query(sql)
            .bind(format!("{}%", keyword))
            .map(AlbumEntity::from_row)
            .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row);
        }

        results
    }
}
