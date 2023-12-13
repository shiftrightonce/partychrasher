use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use sqlx::Column;
use sqlx::Row;

use crate::entity::FromSqliteRow;

#[derive(Debug, Default)]
pub(crate) struct TrackEntity {
    pub(crate) internal_id: i64,
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) path: String,
    pub(crate) metadata: String,
}

impl TrackEntity {
    pub(crate) fn new(id: &str, title: &str, path: &str, metadata: &str) -> Self {
        Self {
            internal_id: 0,
            id: id.to_string(),
            title: title.to_string(),
            path: path.to_string(),
            metadata: metadata.to_string(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InTrackEntityDto {
    pub(crate) title: String,
    pub(crate) path: Option<String>,
    pub(crate) metadata: Option<String>,
}

impl InTrackEntityDto {
    pub(crate) fn new(title: &str, path: Option<String>, metadata: Option<String>) -> Self {
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
            metadata: if entity.metadata.is_empty() {
                None
            } else {
                Some(entity.metadata)
            },
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutTrackEntityDto {
    id: String,
    title: String,
    path: String,
    metadata: String,
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
                "metadata" => entity.metadata = row.get(column.name()),
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
