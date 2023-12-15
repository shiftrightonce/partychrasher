use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use sqlx::Column;
use sqlx::Row;
use symphonia::core::meta::Tag;

use crate::entity::FromSqliteRow;

#[derive(Debug, Default)]
pub(crate) struct TrackEntity {
    pub(crate) internal_id: i64,
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) path: String,
    pub(crate) metadata: TrackMetadata,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InTrackEntityDto {
    pub(crate) title: String,
    pub(crate) path: Option<String>,
    pub(crate) metadata: Option<TrackMetadata>,
}

impl InTrackEntityDto {
    pub(crate) fn new(title: &str, path: Option<String>, metadata: Option<TrackMetadata>) -> Self {
        Self {
            title: title.to_string(),
            path,
            metadata,
        }
    }
}

impl From<TrackEntity> for InTrackEntityDto {
    fn from(entity: TrackEntity) -> Self {
        Self {
            title: entity.title,
            path: if entity.path.is_empty() {
                None
            } else {
                Some(entity.path)
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
    pub(crate) track_number: String,
}

impl From<&[Tag]> for TrackMetadata {
    fn from(tags: &[Tag]) -> Self {
        let mut metadata = Self::default();

        for a_tag in tags.iter() {
            let key = if let Some(k) = a_tag.std_key {
                format!("{:?}", k).to_lowercase()
            } else {
                if a_tag.key.len() > 26 {
                    a_tag.key.to_lowercase().split_at(26).0.to_string()
                } else {
                    a_tag.key.to_lowercase().clone()
                }
            };

            if key.contains("priv:") {
                continue;
            }
            match key.as_str() {
                "title" | "tracktitle" => metadata.title = a_tag.value.to_string(),
                "album" => metadata.album = a_tag.value.to_string(),
                "artist" => metadata.artist = a_tag.value.to_string(),
                "genre" => metadata.genre = a_tag.value.to_string(),
                "tracknumber" => metadata.track_number = a_tag.value.to_string(),
                _ => (),
            }
        }

        metadata
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
    path: String,
    metadata: TrackMetadata,
}

impl From<TrackEntity> for OutTrackEntityDto {
    fn from(entity: TrackEntity) -> Self {
        Self {
            id: entity.id,
            title: entity.title,
            path: entity.path,
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
                "path" => entity.path = row.get(column.name()),
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
