use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use sqlx::Column;
use sqlx::Row;

use crate::entity::FromSqliteRow;

#[derive(Debug, Default)]
pub(crate) struct AlbumArtistEntity {
    pub(crate) album_id: String,
    pub(crate) artist_id: String,
    pub(crate) metadata: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InAlbumArtistEntityDto {
    pub(crate) artist_id: String,
    pub(crate) album_id: String,
    pub(crate) metadata: Option<String>,
}

impl From<AlbumArtistEntity> for InAlbumArtistEntityDto {
    fn from(entity: AlbumArtistEntity) -> Self {
        Self {
            album_id: entity.album_id,
            artist_id: entity.artist_id,
            metadata: if !entity.metadata.is_empty() {
                Some(entity.metadata)
            } else {
                None
            },
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutAlbumArtistEntityDto {
    pub(crate) artist_id: String,
    pub(crate) album_id: String,
    pub(crate) metadata: String,
}

impl From<AlbumArtistEntity> for OutAlbumArtistEntityDto {
    fn from(entity: AlbumArtistEntity) -> Self {
        Self {
            album_id: entity.album_id,
            artist_id: entity.artist_id,
            metadata: entity.metadata,
        }
    }
}

impl Responder for AlbumArtistEntity {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let body = OutAlbumArtistEntityDto::from(self);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&body).unwrap())
    }
}

impl FromSqliteRow for AlbumArtistEntity {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> Option<Self>
    where
        Self: Sized,
    {
        let mut entity = Self::default();

        for column in row.columns() {
            match column.name() {
                "artist_id" => entity.artist_id = row.get(column.name()),
                "album_id" => entity.album_id = row.get(column.name()),
                "metadata" => entity.metadata = row.get(column.name()),
                _ => panic!("New field added to the album_artists table"),
            }
        }

        if entity.album_id.is_empty() {
            None
        } else {
            Some(entity)
        }
    }
}
