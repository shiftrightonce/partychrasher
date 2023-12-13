use futures::stream::TryStreamExt;
use sqlx::sqlite::SqliteRow;
use ulid::Ulid;

use crate::db::{DbConnection, Paginator, PaginatorDirection};
use crate::entity::FromSqliteRow;

use super::{InTrackEntityDto, TrackEntity};

pub(crate) struct TrackRepo {
    pool: DbConnection,
}

impl TrackRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) {
        let sql = r#"CREATE TABLE "tracks" (
    "internal_id"	INTEGER,
	"id"	TEXT,
	"title"	TEXT,
	"path"	TEXT,
	"metadata"	TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);"#;

        _ = sqlx::query(sql).execute(self.pool()).await;
    }

    pub(crate) async fn paginate(&self, paginator: &mut Paginator) -> Vec<TrackEntity> {
        let params = vec![paginator.last_value.clone(), paginator.limit.to_string()];
        let mut rows = Vec::new();
        let sql = match &paginator.direction {
            PaginatorDirection::Next => {
                "select * from tracks where id > ?  order by id asc limit ?"
            }
            PaginatorDirection::Previous => {
                "select * from tracks where id < ?  order by id asc limit ?"
            }
        };

        let mut query = sqlx::query(sql);

        for a_param in params {
            query = query.bind(a_param);
        }

        let mut result_stream = query
            .map(|row: SqliteRow| TrackEntity::from_row(row))
            .fetch(self.pool());

        while let Ok(Some(Some(result))) = result_stream.try_next().await {
            paginator.last_value = result.id.clone();
            rows.push(result)
        }

        rows
    }

    pub(crate) async fn create(&self, entity: InTrackEntityDto) -> Option<TrackEntity> {
        let sql = "INSERT INTO tracks (id , title, path , metadata) values (?, ?, ?, ?)";

        let id = Ulid::new().to_string().to_lowercase();

        if let Err(e) = sqlx::query(sql)
            .bind(&id)
            .bind(entity.title)
            .bind(entity.path.unwrap_or_default())
            .bind(entity.metadata.unwrap_or_default())
            .execute(self.pool())
            .await
        {
            dbg!(e.to_string());
        } else {
            return self.find_by_id(&id).await;
        }

        None
    }

    pub(crate) async fn update(&self, id: &str, entity: InTrackEntityDto) -> Option<TrackEntity> {
        let sql = "UPDATE tracks SET title = ?, path = ?, metadata = ? WHERE id = ?";
        if let Some(existing) = self.find_by_id(id).await {
            if sqlx::query(sql)
                .bind(entity.title)
                .bind(entity.path.unwrap_or(existing.path))
                .bind(entity.metadata.unwrap_or(existing.metadata))
                .execute(self.pool())
                .await
                .is_ok()
            {
                return self.find_by_id(id).await;
            }
        }

        None
    }

    pub(crate) async fn delete(&self, id: &str) -> Option<TrackEntity> {
        let sql = "DELETE FROM tracks WHERE id = ?";
        if let Some(track) = self.find_by_id(id).await {
            if sqlx::query(sql).bind(id).execute(self.pool()).await.is_ok() {
                return Some(track);
            }
        }

        None
    }

    pub(crate) async fn find_by_id(&self, id: &str) -> Option<TrackEntity> {
        let sql = "SELECT * FROM tracks WHERE id = ?";
        if let Ok(row) = sqlx::query(sql)
            .bind(id)
            .map(TrackEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            return row;
        }

        None
    }

    pub(crate) async fn find_by_album_id(&self, album_id: &str) -> Vec<TrackEntity> {
        let sql = "SELECT tracks.internal_id, tracks.path, tracks.id, tracks.title, tracks.metadata FROM album_tracks LEFT JOIN tracks on tracks.id = album_tracks.track_id WHERE album_tracks.album_id = ?";
        let mut results = Vec::new();

        let mut result_stream = sqlx::query(sql)
            .bind(album_id)
            .map(TrackEntity::from_row)
            .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row)
        }

        results
    }

    pub(crate) async fn find_by_playlist_id(&self, playlist_id: &str) -> Vec<TrackEntity> {
        let sql = "SELECT tracks.internal_id, tracks.path, tracks.id, tracks.title, tracks.metadata FROM playlist_tracks LEFT JOIN tracks on tracks.id = album_tracks.track_id WHERE playlist_tracks.playlist_id = ?";
        let mut results = Vec::new();

        let mut result_stream = sqlx::query(sql)
            .bind(playlist_id)
            .map(TrackEntity::from_row)
            .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row)
        }

        results
    }
    pub(crate) async fn find_by_artist_id(&self, artist_id: &str) -> Vec<TrackEntity> {
        let sql = "SELECT tracks.internal_id, tracks.path, tracks.id, tracks.title, tracks.metadata FROM artist_tracks LEFT JOIN tracks on tracks.id = artist_tracks.track_id WHERE artist_tracks.artist_id = ?";
        let mut results = Vec::new();

        let mut result_stream = sqlx::query(sql)
            .bind(artist_id)
            .map(TrackEntity::from_row)
            .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row)
        }

        results
    }

    pub(crate) async fn select_random(&self, limit: i64) -> Vec<TrackEntity> {
        let sql = "SELECT * FROM tracks ORDER BY RANDOM() LIMIT ?";
        let mut results = Vec::new();

        let mut result_stream = sqlx::query(sql)
            .bind(limit)
            .map(TrackEntity::from_row)
            .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row)
        }

        results
    }
}
