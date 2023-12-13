use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use sqlx::Column;
use sqlx::Row;

use crate::entity::FromSqliteRow;

#[derive(Debug, Default)]
pub(crate) struct ArtistTrackEntity {
    pub(crate) artist_id: String,
    pub(crate) track_id: String,
    pub(crate) is_feature: bool,
    pub(crate) metadata: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InArtistTrackEntityDto {
    pub(crate) artist_id: String,
    pub(crate) track_id: String,
    pub(crate) is_feature: bool,
    pub(crate) metadata: Option<String>,
}

impl From<ArtistTrackEntity> for InArtistTrackEntityDto {
    fn from(entity: ArtistTrackEntity) -> Self {
        Self {
            artist_id: entity.artist_id,
            track_id: entity.track_id,
            is_feature: entity.is_feature,
            metadata: if entity.metadata.is_empty() {
                None
            } else {
                Some(entity.metadata)
            },
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutArtistTrackEntityDto {
    pub(crate) artist_id: String,
    pub(crate) track_id: String,
    pub(crate) is_feature: bool,
    pub(crate) metadata: String,
}

impl From<ArtistTrackEntity> for OutArtistTrackEntityDto {
    fn from(entity: ArtistTrackEntity) -> Self {
        Self {
            artist_id: entity.artist_id,
            track_id: entity.track_id,
            is_feature: entity.is_feature,
            metadata: entity.metadata,
        }
    }
}

impl Responder for ArtistTrackEntity {
    type Body = BoxBody;
    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = OutArtistTrackEntityDto::from(self);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&body).unwrap())
    }
}

impl FromSqliteRow for ArtistTrackEntity {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> Option<Self>
    where
        Self: Sized,
    {
        let mut entity = Self::default();

        for column in row.columns() {
            match column.name() {
                "artist_id" => entity.artist_id = row.get(column.name()),
                "track_id" => entity.track_id = row.get(column.name()),
                "is_feature" => entity.is_feature = row.get(column.name()),
                "metadata" => entity.metadata = row.get(column.name()),
                _ => panic!("New field added to the artist_tracks table"),
            }
        }

        if entity.artist_id.is_empty() {
            None
        } else {
            Some(entity)
        }
    }
}
