use std::collections::HashMap;

use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use sqlx::Column;
use sqlx::Row;

use crate::entity::media::MediaMetadata;
use crate::entity::FromSqliteRow;

#[derive(Debug, Default)]
pub(crate) struct TrackEntity {
    pub(crate) internal_id: i64,
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) media_id: String,
    pub(crate) metadata: TrackMetadata,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InTrackEntityDto {
    pub(crate) title: String,
    pub(crate) media_id: Option<String>,
    pub(crate) metadata: Option<TrackMetadata>,
}

impl InTrackEntityDto {
    pub(crate) fn new(
        title: &str,
        media_id: Option<String>,
        metadata: Option<TrackMetadata>,
    ) -> Self {
        Self {
            title: title.to_string(),
            media_id,
            metadata,
        }
    }
}

impl From<TrackEntity> for InTrackEntityDto {
    fn from(entity: TrackEntity) -> Self {
        Self {
            title: entity.title,
            media_id: if entity.media_id.is_empty() {
                None
            } else {
                Some(entity.media_id)
            },
            metadata: Some(entity.metadata),
        }
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct TrackMetadata {
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) album: String,
    pub(crate) genre: String,
    pub(crate) track: u32,
    pub(crate) disk: u32,
    pub(crate) year: u32,
    pub(crate) pictures: HashMap<String, String>,
}

impl From<&MediaMetadata> for TrackMetadata {
    fn from(entity: &MediaMetadata) -> Self {
        Self {
            title: entity.title.clone(),
            artist: entity.artist.clone(),
            album: entity.album.clone(),
            genre: entity.genre.clone(),
            track: entity.track,
            disk: entity.disk,
            year: entity.year,
            pictures: entity.pictures.clone(),
        }
    }
}

impl ToString for TrackMetadata {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutTrackEntityDto {
    id: String,
    title: String,
    metadata: TrackMetadata,
}

impl From<TrackEntity> for OutTrackEntityDto {
    fn from(entity: TrackEntity) -> Self {
        Self {
            id: entity.id,
            title: entity.title,
            metadata: entity.metadata,
        }
    }
}

impl Responder for TrackEntity {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = OutTrackEntityDto::from(self);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&body).unwrap())
    }
}

impl FromSqliteRow for TrackEntity {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> Option<Self>
    where
        Self: Sized,
    {
        let mut entity = Self::default();

        for column in row.columns() {
            match column.name() {
                "internal_id" => entity.internal_id = row.get(column.name()),
                "id" => entity.id = row.get(column.name()),
                "title" => entity.title = row.get(column.name()),
                "media_id" => entity.media_id = row.get(column.name()),
                "metadata" => {
                    let value: String = row.get(column.name());
                    if let Ok(metadata) = serde_json::from_str(value.as_str()) {
                        entity.metadata = metadata;
                    }
                }

                _ => panic!("New field added to the tracks table"),
            }
        }

        if entity.internal_id > 0 {
            Some(entity)
        } else {
            None
        }
    }
}
