#![allow(dead_code)]

use futures::stream::TryStreamExt;
use orsomafo::Dispatchable;
use sqlx::sqlite::SqliteRow;
use ulid::Ulid;

use crate::{
    db::{DbConnection, Paginator, PaginatorDirection},
    entity::FromSqliteRow,
};

use super::{AlbumAddedEvent, AlbumDeletedEvent, AlbumEntity, AlbumUpdatedEvent, InAlbumEntityDto};

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
	"id"	TEXT NOT NULL UNIQUE,
	"title"	TEXT,
    "year" INTEGER, 
	"metadata"	TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT),
    UNIQUE("title", "year")
);"#;

        _ = sqlx::query(sql).execute(self.pool()).await;
    }

    pub(crate) async fn create(&self, album: InAlbumEntityDto) -> Option<AlbumEntity> {
        let sql = "INSERT OR IGNORE INTO albums (id, title, metadata, year) values (?, ?, ?, ?)";

        let id = Ulid::new().to_string().to_lowercase();

        if sqlx::query(sql)
            .bind(&id)
            .bind(album.title)
            .bind(album.metadata.unwrap_or_default().to_string())
            .bind(album.year.unwrap_or_default())
            .execute(self.pool())
            .await
            .is_ok()
        {
            let result = self.find_by_id(&id).await;

            // Dispatch album added event
            if result.is_some() {
                (AlbumAddedEvent {
                    album_id: result.as_ref().unwrap().id.clone(),
                })
                .dispatch_event();
            }

            return result;
        }
        None
    }

    pub(crate) async fn update(&self, id: &str, album: InAlbumEntityDto) -> Option<AlbumEntity> {
        let sql = "UPDATE albums set title =?, metadata = ?, year = ? WHERE id = ?";
        if let Some(existing) = self.find_by_id(id).await {
            if sqlx::query(sql)
                .bind(album.title)
                .bind(album.metadata.unwrap_or(existing.metadata).to_string())
                .bind(album.year.unwrap_or(existing.year))
                .bind(id)
                .execute(self.pool())
                .await
                .is_ok()
            {
                let result = self.find_by_id(id).await;

                // Dispatch album updated event
                if result.is_none() {
                    (AlbumUpdatedEvent {
                        album_id: result.as_ref().unwrap().id.clone(),
                    })
                    .dispatch_event();
                }
                return result;
            }
        }

        None
    }

    pub(crate) async fn delete(&self, id: &str) -> Option<AlbumEntity> {
        if let Some(entity) = self.find_by_id(id).await {
            if sqlx::query("DELETE FROM albums WHERE id = ?")
                .bind(id)
                .execute(self.pool())
                .await
                .is_ok()
            {
                // Dispatch album deleted event
                (AlbumDeletedEvent {
                    album_id: entity.id.clone(),
                })
                .dispatch_event();

                return Some(entity);
            }
        }

        None
    }

    pub(crate) async fn paginate(&self, paginator: &mut Paginator) -> Vec<AlbumEntity> {
        let params = vec![paginator.last_value.clone(), paginator.limit.to_string()];
        let mut rows = Vec::new();
        let sql = match &paginator.direction {
            PaginatorDirection::Next => {
                "select * from albums where id > ?  order by id asc limit ?"
            }
            PaginatorDirection::Previous => {
                "select * from albums where id < ?  order by id asc limit ?"
            }
        };

        let mut query = sqlx::query(sql);

        for a_param in params {
            query = query.bind(a_param);
        }

        let mut result_stream = query
            .map(|row: SqliteRow| AlbumEntity::from_row(row))
            .fetch(self.pool());

        while let Ok(Some(Some(result))) = result_stream.try_next().await {
            paginator.last_value = result.id.clone();
            rows.push(result)
        }

        rows
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

    pub(crate) async fn find_by_track_id(&self, track_id: &str) -> Vec<AlbumEntity> {
        let sql = "SELECT albums.internal_id, albums.id, albums.title, albums.metadata FROM album_tracks LEFT JOIN albums on albums.id = album_tracks.album_id WHERE album_tracks.track_id = ?";
        let mut results = Vec::new();

        let mut result_stream = sqlx::query(sql)
            .bind(track_id)
            .map(AlbumEntity::from_row)
            .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row)
        }

        results
    }

    pub(crate) async fn find_by_artist_id(&self, artist_id: &str) -> Vec<AlbumEntity> {
        let sql = "SELECT albums.internal_id, albums.id, albums.title, albums.metadata FROM album_artists LEFT JOIN albums on albums.id = album_artists.album_id WHERE album_artists.artist_id = ?";
        let mut results = Vec::new();

        let mut result_stream = sqlx::query(sql)
            .bind(artist_id)
            .map(AlbumEntity::from_row)
            .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row)
        }

        results
    }

    pub(crate) async fn search(&self, keyword: &str) -> Vec<AlbumEntity> {
        let sql = "SELECT * FROM albums WHERE title LIKE ? LIMIT 100";
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
