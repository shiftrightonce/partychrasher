use futures::stream::TryStreamExt;
use orsomafo::Dispatchable;
use sqlx::sqlite::SqliteRow;
use ulid::Ulid;

use crate::{
    db::{DbConnection, Paginator, PaginatorDirection},
    entity::FromSqliteRow,
};

use super::{ArtistAddedEvent, ArtistEntity, ArtistUpdatedEvent, InArtistEntityDto};

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
            let result = self.find_by_id(&id).await;

            // Dispatch artist created event
            if result.is_some() {
                (ArtistAddedEvent {
                    artist_id: result.as_ref().unwrap().id.clone(),
                })
                .dispatch_event();
            }

            return result;
        }

        None
    }

    pub(crate) async fn update(&self, id: &str, artist: InArtistEntityDto) -> Option<ArtistEntity> {
        let sql = "UPDATE artists SET name = ?, metadata = ? WHERE id = ?";

        if let Some(existing) = self.find_by_id(id).await {
            if sqlx::query(sql)
                .bind(artist.name)
                .bind(artist.metadata.unwrap_or(existing.metadata))
                .execute(self.pool())
                .await
                .is_ok()
            {
                let result = self.find_by_id(id).await;

                // Dispatch artist updated event
                if result.is_some() {
                    (ArtistUpdatedEvent {
                        artist_id: result.as_ref().unwrap().id.clone(),
                    })
                    .dispatch_event();
                }

                return result;
            }
        }

        None
    }

    pub async fn create_or_update(&self, artist: InArtistEntityDto) -> Option<ArtistEntity> {
        if let Ok(Some(existing)) = sqlx::query("SELECT * FROM artists WHERE name = ?")
            .bind(&artist.name)
            .map(ArtistEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            self.update(&existing.id, artist).await
        } else {
            self.create(artist).await
        }
    }

    pub(crate) async fn paginate(&self, paginator: &mut Paginator) -> Vec<ArtistEntity> {
        let params = vec![paginator.last_value.clone(), paginator.limit.to_string()];
        let mut rows = Vec::new();
        let sql = match &paginator.direction {
            PaginatorDirection::Next => {
                "select * from artists where id > ?  order by id asc limit ?"
            }
            PaginatorDirection::Previous => {
                "select * from artists where id < ?  order by id asc limit ?"
            }
        };

        let mut query = sqlx::query(sql);

        for a_param in params {
            query = query.bind(a_param);
        }

        let mut result_stream = query
            .map(|row: SqliteRow| ArtistEntity::from_row(row))
            .fetch(self.pool());

        while let Ok(Some(Some(result))) = result_stream.try_next().await {
            paginator.last_value = result.id.clone();
            rows.push(result)
        }

        rows
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

    pub(crate) async fn find_by_track_id(&self, track_id: &str) -> Vec<ArtistEntity> {
        let sql = "SELECT artists.internal_id, artists.id, artists.name, artists.metadata FROM artist_tracks LEFT JOIN artists on artists.id = artist_tracks.artist_id WHERE artist_tracks.track_id = ?";
        let mut results = Vec::new();

        let mut result_stream = sqlx::query(sql)
            .bind(track_id)
            .map(ArtistEntity::from_row)
            .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row)
        }

        results
    }

    pub(crate) async fn find_by_album_id(&self, album_id: &str) -> Vec<ArtistEntity> {
        let sql = "SELECT artists.internal_id, artists.id, artists.name, artists.metadata FROM album_artists LEFT JOIN artists on artists.id = album_artists.artist_id WHERE album_artists.album_id = ?";
        let mut results = Vec::new();

        let mut result_stream = sqlx::query(sql)
            .bind(album_id)
            .map(ArtistEntity::from_row)
            .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row)
        }

        results
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
