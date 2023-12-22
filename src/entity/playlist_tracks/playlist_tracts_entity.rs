use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use sqlx::Column;
use sqlx::Row;

use crate::entity::FromSqliteRow;

#[derive(Debug, Default)]
pub(crate) struct PlaylistTrackEntity {
    pub(crate) internal_id: i64,
    pub(crate) playlist_id: String,
    pub(crate) track_id: String,
    pub(crate) metadata: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InPlaylistTrackEntityDto {
    pub(crate) playlist_id: String,
    pub(crate) track_id: String,
    pub(crate) metadata: Option<String>,
}

impl From<PlaylistTrackEntity> for InPlaylistTrackEntityDto {
    fn from(entity: PlaylistTrackEntity) -> Self {
        Self {
            playlist_id: entity.playlist_id,
            track_id: entity.track_id,
            metadata: if !entity.metadata.is_empty() {
                Some(entity.metadata)
            } else {
                None
            },
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutPlaylistTrackEntityDto {
    pub(crate) order_number: i64,
    pub(crate) playlist_id: String,
    pub(crate) track_id: String,
    pub(crate) metadata: String,
}

impl From<PlaylistTrackEntity> for OutPlaylistTrackEntityDto {
    fn from(entity: PlaylistTrackEntity) -> Self {
        Self {
            order_number: entity.internal_id,
            playlist_id: entity.playlist_id,
            track_id: entity.track_id,
            metadata: entity.metadata,
        }
    }
}

impl Responder for PlaylistTrackEntity {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = OutPlaylistTrackEntityDto::from(self);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&body).unwrap())
    }
}

impl FromSqliteRow for PlaylistTrackEntity {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> Option<Self>
    where
        Self: Sized,
    {
        let mut entity = Self::default();

        for column in row.columns() {
            match column.name() {
                "playlist_id" => entity.playlist_id = row.get(column.name()),
                "track_id" => entity.track_id = row.get(column.name()),
                "internal_id" => entity.internal_id = row.get(column.name()),
                "metadata" => entity.metadata = row.get(column.name()),
                _ => panic!("New field added to the playlist_tracks table"),
            }
        }

        if entity.playlist_id.is_empty() {
            None
        } else {
            Some(entity)
        }
    }
}
