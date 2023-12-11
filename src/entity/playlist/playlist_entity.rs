use actix_web::{body::BoxBody, http::header::ContentType, HttpResponse, Responder};
use sqlx::{Column, Row};
use symphonia::core::io::vlc::CodebookEntry;
use ulid::Ulid;

use crate::entity::{client::ClientEntity, FromSqliteRow};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct PlaylistEntity {
    pub(crate) internal_id: i64,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
}

impl Default for PlaylistEntity {
    fn default() -> Self {
        Self {
            internal_id: 0,
            id: String::new(),
            name: Ulid::new().to_string(),
            description: String::new(),
        }
    }
}

impl PlaylistEntity {
    pub(crate) fn new(name: &str, description: Option<String>) -> Self {
        Self {
            name: name.to_string(),
            description: description.unwrap_or_default(),
            ..Self::default()
        }
    }
}

impl FromSqliteRow for PlaylistEntity {
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
                "description" => entity.description = row.get(column.name()),
                _ => panic!("new field added to the playlist table and must be handled"),
            }
        }

        if entity.internal_id > 0 {
            Some(entity)
        } else {
            None
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InPlaylistEntityDto {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
}

impl From<PlaylistEntity> for InPlaylistEntityDto {
    fn from(value: PlaylistEntity) -> Self {
        Self {
            name: value.name,
            description: Some(value.description),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutPlaylistEntityDto {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
}

impl From<PlaylistEntity> for OutPlaylistEntityDto {
    fn from(value: PlaylistEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
        }
    }
}

impl Responder for PlaylistEntity {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let body = OutPlaylistEntityDto::from(self);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&body).unwrap())
    }
}
