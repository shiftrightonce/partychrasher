use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use ulid::Ulid;

use crate::db::DbConnection;
use crate::entity::FromSqliteRow;

use super::{InMediaEntityDto, MediaEntity};

pub(crate) struct MediaRepo {
    pool: DbConnection,
}

impl MediaRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) {
        let sql = r#"CREATE TABLE IF NOT EXISTS "media" (
    "internal_id"	INTEGER,
	"id"	TEXT NOT NULL UNIQUE,
	"filename"	TEXT NOT NULL,
	"media_type"	TEXT NOT NULL,
	"path"	TEXT NOT NULL,
	"metadata"	TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT),
  UNIQUE("filename", "path")
);"#;

        if let Err(e) = sqlx::query(sql).execute(self.pool()).await {
            dbg!(e);
        }
    }

    pub(crate) async fn create(&self, entity: InMediaEntityDto) -> Option<MediaEntity> {
        let sql = "INSERT INTO media (id , filename , path , metadata, media_type) values (?, ?, ?, ?, ?)";

        let id = Ulid::new().to_string().to_lowercase();

        if let Err(e) = sqlx::query(sql)
            .bind(&id)
            .bind(entity.filename)
            .bind(entity.path.unwrap_or_default())
            .bind(entity.metadata.unwrap_or_default().to_string())
            .bind(entity.media_type.unwrap_or_default().to_string())
            .execute(self.pool())
            .await
        {
            dbg!(e.to_string());
        } else {
            return self.find_by_id(&id).await;
        }

        None
    }

    pub(crate) async fn create_or_update(&self, entity: InMediaEntityDto) -> Option<MediaEntity> {
        if let Ok(id) = sqlx::query(r#"SELECT "id" FROM media WHERE filename = ? AND path = ?"#)
            .bind(&entity.filename)
            .bind(&entity.path)
            .map(|row: SqliteRow| row.get::<String, &str>("id"))
            .fetch_one(self.pool())
            .await
        {
            self.update(&id, entity).await
        } else {
            self.create(entity).await
        }
    }

    pub(crate) async fn update(&self, id: &str, entity: InMediaEntityDto) -> Option<MediaEntity> {
        let sql =
            "UPDATE media SET filename = ?, path = ?, media_type = ?, metadata = ? WHERE id = ?";
        if let Some(existing) = self.find_by_id(id).await {
            if sqlx::query(sql)
                .bind(entity.filename)
                .bind(entity.path.unwrap_or(existing.path))
                .bind(entity.media_type.unwrap_or(existing.media_type).to_string())
                .bind(entity.metadata.unwrap_or(existing.metadata).to_string())
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

    pub(crate) async fn find_by_id(&self, id: &str) -> Option<MediaEntity> {
        let sql = "SELECT * FROM media WHERE id = ?";
        if let Ok(row) = sqlx::query(sql)
            .bind(id)
            .map(MediaEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            return row;
        }

        None
    }

    pub(crate) async fn find_by_filename_and_path(
        &self,
        filename: &str,
        path: &str,
    ) -> Option<MediaEntity> {
        let sql = "SELECT * FROM media WHERE filename = ? AND path = ?";
        if let Ok(row) = sqlx::query(sql)
            .bind(filename)
            .bind(path)
            .map(MediaEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            return row;
        }

        None
    }

    pub(crate) async fn find_media_by_track(&self, track_id: &str) -> Option<MediaEntity> {
        let sql = r#"SELECT media.internal_id as internal_id, media.id as "id", media.filename as filename, media.media_type as media_type, media.path as path  FROM media LEFT JOIN tracks on tracks.media_id = media.id WHERE tracks.id = ?"#;
        if let Ok(row) = sqlx::query(sql)
            .bind(track_id)
            .map(MediaEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            return row;
        }

        None
    }
}
