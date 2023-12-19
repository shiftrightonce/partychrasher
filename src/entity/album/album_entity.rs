use std::collections::HashMap;

use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use sqlx::Column;
use sqlx::Row;

use crate::entity::track::TrackMetadata;
use crate::entity::FromSqliteRow;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct AlbumMetadata {
    pictures: HashMap<String, String>,
}

impl ToString for AlbumMetadata {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl From<&TrackMetadata> for AlbumMetadata {
    fn from(value: &TrackMetadata) -> Self {
        Self {
            pictures: value.pictures.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct AlbumEntity {
    pub(crate) internal_id: i64,
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) year: u32,
    pub(crate) metadata: AlbumMetadata,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InAlbumEntityDto {
    pub(crate) title: String,
    pub(crate) year: Option<u32>,
    pub(crate) metadata: Option<AlbumMetadata>,
}

impl From<AlbumEntity> for InAlbumEntityDto {
    fn from(value: AlbumEntity) -> Self {
        Self {
            title: value.title,
            year: if value.year == 0 {
                None
            } else {
                Some(value.year)
            },
            metadata: Some(value.metadata),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutAlbumEntityDto {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) year: u32,
    pub(crate) metadata: AlbumMetadata,
}

impl From<AlbumEntity> for OutAlbumEntityDto {
    fn from(value: AlbumEntity) -> Self {
        Self {
            id: value.id,
            title: value.title,
            year: value.year,
            metadata: value.metadata,
        }
    }
}

impl Responder for AlbumEntity {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let body = OutAlbumEntityDto::from(self);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&body).unwrap())
    }
}

impl FromSqliteRow for AlbumEntity {
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
                "metadata" => {
                    let value: String = row.get(column.name());
                    if let Ok(metadata) = serde_json::from_str(value.as_str()) {
                        entity.metadata = metadata
                    }
                }
                "year" => entity.year = row.get(column.name()),
                _ => panic!("New field added to the albums table"),
            }
        }

        if entity.internal_id > 0 {
            Some(entity)
        } else {
            None
        }
    }
}
