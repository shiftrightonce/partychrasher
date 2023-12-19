use futures_util::TryStreamExt;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use ulid::Ulid;

use crate::{
    db::{DbConnection, Paginator, PaginatorDirection},
    entity::FromSqliteRow,
};

use super::{InPlaylistEntityDto, PlaylistEntity};

pub(crate) struct PlaylistRepo {
    pool: DbConnection,
}

impl PlaylistRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) -> Option<PlaylistEntity> {
        let sql = r#"CREATE TABLE "playlists" (
	"internal_id"	INTEGER,
	"id"	TEXT NOT NULL UNIQUE,
	"is_default"	NUMBER DEFAULT 0,
	"name"	TEXT NOT NULL UNIQUE,
	"description"	TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);"#;

        if sqlx::query(sql).execute(self.pool()).await.is_ok() && !self.has_default_playlist().await
        {
            return self.create_default_playlist().await;
        }
        None
    }

    pub(crate) async fn has_default_playlist(&self) -> bool {
        let sql = r#"SELECT COUNT("internal_id") AS total FROM playlists where name = ?"#;

        match sqlx::query(sql)
            .bind("default playlist")
            .map(|row: SqliteRow| row.get::<i64, &str>("total"))
            .fetch_one(self.pool())
            .await
        {
            Ok(total) => total > 0,
            _ => false,
        }
    }

    pub(crate) async fn create_default_playlist(&self) -> Option<PlaylistEntity> {
        let name = "default playlist";
        let playlist = PlaylistEntity::new(
            name,
            true,
            Some("Default playlist generated by the app".to_string()),
        );

        let default = self.create(playlist.into()).await;

        if default.is_some() {
            if let Ok(mut content) = tokio::fs::read_to_string("./.env").await {
                content = content.replace("playlist_id", &default.as_ref().unwrap().id);
                _ = tokio::fs::write("./.env", content).await;
            }
        }

        default
    }

    pub(crate) async fn create(&self, playlist: InPlaylistEntityDto) -> Option<PlaylistEntity> {
        let sql =
            r#"INSERT INTO playlists (id, name, description, is_default) values (?, ?, ?, ?)"#;

        let id = Ulid::new().to_string().to_lowercase();

        if sqlx::query(sql)
            .bind(&id)
            .bind(playlist.name)
            .bind(playlist.description.unwrap_or_default())
            .bind(playlist.is_default.unwrap_or_default())
            .execute(self.pool())
            .await
            .is_ok()
        {
            let result = self.find_by_id(&id).await;
            if let Some(playlist) = &result {
                self.clean_existing_default(&playlist.id, playlist.is_default)
                    .await;
            }

            result
        } else {
            None
        }
    }

    pub(crate) async fn update(
        &self,
        id: &str,
        playlist: InPlaylistEntityDto,
    ) -> Option<PlaylistEntity> {
        let sql = "UPDATE playlists set name = ?, description = ?, is_default = ?";

        if let Some(existing) = self.find_by_id(id).await {
            _ = sqlx::query(sql)
                .bind(playlist.name)
                .bind(playlist.description.unwrap_or(existing.description))
                .bind(playlist.is_default.unwrap_or(existing.is_default))
                .execute(self.pool())
                .await;
            self.clean_existing_default(id, playlist.is_default.unwrap_or_default())
                .await;
            return self.find_by_id(id).await;
        }

        None
    }

    pub(crate) async fn delete(&self, id: &str) -> Option<PlaylistEntity> {
        if let Some(existing) = self.find_by_id(id).await {
            if let Err(e) = sqlx::query("DELETE FROM playlists WHERE id = ? and is_default = 0")
                .bind(id)
                .execute(self.pool())
                .await
            {
                println!("error deleting playlist: {:?}", e);
            } else {
                return Some(existing);
            }
        }

        None
    }

    pub(crate) async fn paginate(&self, paginator: &mut Paginator) -> Vec<PlaylistEntity> {
        let params = vec![paginator.last_value.clone(), paginator.limit.to_string()];
        let mut rows = Vec::new();
        let sql = match &paginator.direction {
            PaginatorDirection::Next => {
                "select * from playlists where id > ?  order by id asc limit ?"
            }
            PaginatorDirection::Previous => {
                "select * from playlists where id < ?  order by id asc limit ?"
            }
        };

        let mut query = sqlx::query(sql);

        for a_param in params {
            query = query.bind(a_param);
        }

        let mut result_stream = query
            .map(|row: SqliteRow| PlaylistEntity::from_row(row))
            .fetch(self.pool());

        while let Ok(Some(Some(result))) = result_stream.try_next().await {
            paginator.last_value = result.id.clone();
            rows.push(result)
        }

        rows
    }

    pub(crate) async fn find_by_id(&self, id: &str) -> Option<PlaylistEntity> {
        if let Ok(row) = sqlx::query("SELECT * FROM playlists WHERE id = ? ")
            .bind(id)
            .map(PlaylistEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            row
        } else {
            None
        }
    }

    pub(crate) async fn get_default_playlist(&self) -> Option<PlaylistEntity> {
        if let Ok(row) = sqlx::query("SELECT * FROM playlists WHERE is_default = ? ")
            .bind(1_i64)
            .map(PlaylistEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            row
        } else {
            None
        }
    }

    async fn clean_existing_default(&self, id: &str, is_default: bool) {
        if is_default {
            _ = sqlx::query("UPDATE playlists set is_default = 0 WHERE is_default = 1 AND id != ?")
                .bind(id)
                .execute(self.pool())
                .await;
        }
    }
}
