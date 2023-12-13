use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use sqlx::Column;
use sqlx::Row;

use crate::entity::FromSqliteRow;

#[derive(Debug, Default)]
pub(crate) struct AlbumTrackEntity {
    pub(crate) album_id: String,
    pub(crate) track_id: String,
    pub(crate) metadata: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InAlbumTrackEntityDto {
    pub(crate) album_id: String,
    pub(crate) track_id: String,
    pub(crate) metadata: Option<String>,
}

impl From<AlbumTrackEntity> for InAlbumTrackEntityDto {
    fn from(entity: AlbumTrackEntity) -> Self {
        Self {
            album_id: entity.album_id,
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
pub(crate) struct OutAlbumTrackEntityDto {
    pub(crate) album_id: String,
    pub(crate) track_id: String,
    pub(crate) metadata: String,
}

impl From<AlbumTrackEntity> for OutAlbumTrackEntityDto {
    fn from(entity: AlbumTrackEntity) -> Self {
        Self {
            album_id: entity.album_id,
            track_id: entity.track_id,
            metadata: entity.metadata,
        }
    }
}

impl Responder for AlbumTrackEntity {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let body = OutAlbumTrackEntityDto::from(self);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&body).unwrap())
    }
}

impl FromSqliteRow for AlbumTrackEntity {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> Option<Self>
    where
        Self: Sized,
    {
        let mut entity = Self::default();

        for column in row.columns() {
            match column.name() {
                "track_id" => entity.track_id = row.get(column.name()),
                "album_id" => entity.album_id = row.get(column.name()),
                "metadata" => entity.metadata = row.get(column.name()),
                _ => panic!("New field added to the album_tracks table"),
            }
        }

        if entity.album_id.is_empty() {
            None
        } else {
            Some(entity)
        }
    }
}
