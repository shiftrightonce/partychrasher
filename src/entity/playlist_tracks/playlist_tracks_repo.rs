use futures::stream::TryStreamExt;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use ulid::Ulid;

use crate::{db::DbConnection, entity::FromSqliteRow, queue_manager::setup_queue_manager};

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
CREATE TABLE "playlist_tracts" (
    "track_id"	TEXT NOT NULL,
    "playlist_id"	TEXT NOT NULL,
    "metadata" TEXT,
    UNIQUE("track_id","playlist_id")
);
       "#;

        _ = sqlx::query(sql).execute(self.pool()).await;
    }
}
