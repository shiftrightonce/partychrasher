use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use sqlx::Column;
use sqlx::Row;

use crate::entity::FromSqliteRow;

#[derive(Debug, Default)]
pub(crate) struct ArtistEntity {
    pub(crate) internal_id: i64,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) metadata: String,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InArtistEntityDto {
    pub(crate) name: String,
    pub(crate) metadata: Option<String>,
}

impl From<ArtistEntity> for InArtistEntityDto {
    fn from(entity: ArtistEntity) -> Self {
        Self {
            name: entity.name,
            metadata: if entity.metadata.is_empty() {
                None
            } else {
                Some(entity.metadata)
            },
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutArtistEntityDto {
    id: String,
    name: String,
    metadata: String,
}

impl From<ArtistEntity> for OutArtistEntityDto {
    fn from(entity: ArtistEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            metadata: entity.metadata,
        }
    }
}

impl Responder for ArtistEntity {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = OutArtistEntityDto::from(self);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&body).unwrap())
    }
}

impl FromSqliteRow for ArtistEntity {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> Option<Self>
    where
        Self: Sized,
    {
        let mut entity = Self::default();

        for column in row.columns() {
            match column.name() {
                "internal_id" => entity.internal_id = row.get(column.name()),
                "id" => entity.id = row.get(column.name()),
                "name" => entity.name = row.get(column.name()),
                "metadata" => entity.metadata = row.get(column.name()),
                _ => panic!("New field added to the artists table"),
            }
        }

        if entity.internal_id > 0 {
            Some(entity)
        } else {
            None
        }
    }
}
