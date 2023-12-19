use std::collections::HashMap;

use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use lofty::{Accessor, Tag};
use sqlx::Column;
use sqlx::Row;

use crate::entity::track::{InTrackEntityDto, TrackMetadata};
use crate::entity::FromSqliteRow;

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, PartialOrd)]
pub(crate) enum MediaType {
    #[serde(rename(deserialize = "video", serialize = "video"))]
    Video,
    #[serde(rename(deserialize = "audio", serialize = "audio"))]
    Audio,
    #[serde(rename(deserialize = "photo", serialize = "photo"))]
    Photo,
    #[serde(rename(deserialize = "unknown", serialize = "unknown"))]
    Unknown,
}

impl Default for MediaType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<String> for MediaType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "audio" => Self::Audio,
            "video" => Self::Video,
            "photo" => Self::Photo,
            "unknown" => Self::Unknown,
            _ => Self::default(),
        }
    }
}

impl ToString for MediaType {
    fn to_string(&self) -> String {
        match self {
            Self::Audio => "audio".to_string(),
            Self::Video => "video".to_owned(),
            Self::Photo => "photo".to_string(),
            Self::Unknown => "unknown".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct MediaEntity {
    pub(crate) internal_id: i64,
    pub(crate) id: String,
    pub(crate) media_type: MediaType,
    pub(crate) filename: String,
    pub(crate) path: String,
    pub(crate) metadata: MediaMetadata,
}

impl MediaEntity {
    pub(crate) fn is_audio(&self) -> bool {
        self.media_type == MediaType::Audio
    }
}

impl TryInto<InTrackEntityDto> for MediaEntity {
    type Error = String;
    fn try_into(self) -> Result<InTrackEntityDto, Self::Error> {
        if self.is_audio() {
            let title = if self.metadata.title.is_empty() {
                self.filename.clone()
            } else {
                self.metadata.title.clone()
            };
            Ok(InTrackEntityDto::new(
                &title,
                Some(self.id.clone()),
                Some(TrackMetadata::from(&self.metadata)),
            ))
        } else {
            Err("media file is not an audio file".to_string())
        }
    }
}

impl TryFrom<&MediaEntity> for InTrackEntityDto {
    type Error = String;
    fn try_from(value: &MediaEntity) -> Result<Self, Self::Error> {
        if value.is_audio() {
            let title = if value.metadata.title.is_empty() {
                value.filename.clone()
            } else {
                value.metadata.title.clone()
            };
            Ok(Self::new(
                &title,
                Some(value.id.clone()),
                Some(TrackMetadata::from(&value.metadata)),
            ))
        } else {
            Err("media file is not an audio file".to_string())
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InMediaEntityDto {
    pub(crate) filename: String,
    pub(crate) media_type: Option<MediaType>,
    pub(crate) path: Option<String>,
    pub(crate) metadata: Option<MediaMetadata>,
}

impl InMediaEntityDto {
    pub(crate) fn new_from_str(
        filename: &str,
        extension: &str,
        path: Option<String>,
        metadata: Option<MediaMetadata>,
    ) -> Self {
        Self {
            filename: filename.to_string(),
            media_type: Some(match extension.trim().to_lowercase().as_str() {
                "mp3" | "aac" | "m4a" | "wav" | "ogg" | "wma" | "webm" | "flac" => MediaType::Audio,
                "mp4" | "avi" => MediaType::Video,
                "jpg" | "png" | "gif" => MediaType::Photo,
                _ => MediaType::default(),
            }),
            path,
            metadata,
        }
    }
}

impl From<MediaEntity> for InMediaEntityDto {
    fn from(entity: MediaEntity) -> Self {
        Self {
            filename: entity.filename,
            media_type: Some(entity.media_type),
            path: if entity.path.is_empty() {
                None
            } else {
                Some(entity.path)
            },
            metadata: Some(entity.metadata),
        }
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize, Clone)]
pub(crate) struct MediaMetadata {
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) album: String,
    pub(crate) genre: String,
    pub(crate) track: u32,
    pub(crate) disk: u32,
    pub(crate) year: u32,
    pub(crate) pictures: HashMap<String, String>,
}

impl From<&Tag> for MediaMetadata {
    fn from(tag: &Tag) -> Self {
        let mut metadata = Self::default();
        if let Some(title) = tag.title() {
            metadata.title = title.to_string();
        }
        if let Some(artist) = tag.artist() {
            metadata.artist = artist.to_string();
        }
        if let Some(album) = tag.album() {
            metadata.album = album.to_string();
        }
        if let Some(track) = tag.track() {
            metadata.track = track;
        }
        if let Some(disk) = tag.disk() {
            metadata.disk = disk;
        }
        if let Some(year) = tag.year() {
            metadata.year = year;
        }

        if let Some(genre) = tag.genre() {
            metadata.genre = genre.to_string();
        }
        metadata
    }
}

impl ToString for MediaMetadata {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutMediaEntityDto {
    id: String,
    filename: String,
    media_type: MediaType,
    path: String,
    metadata: MediaMetadata,
}

impl From<MediaEntity> for OutMediaEntityDto {
    fn from(entity: MediaEntity) -> Self {
        Self {
            id: entity.id,
            filename: entity.filename,
            media_type: entity.media_type,
            path: entity.path,
            metadata: entity.metadata,
        }
    }
}

impl Responder for MediaEntity {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = OutMediaEntityDto::from(self);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&body).unwrap())
    }
}

impl FromSqliteRow for MediaEntity {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> Option<Self>
    where
        Self: Sized,
    {
        let mut entity = Self::default();

        for column in row.columns() {
            match column.name() {
                "internal_id" => entity.internal_id = row.get(column.name()),
                "id" => entity.id = row.get(column.name()),
                "filename" => entity.filename = row.get(column.name()),
                "media_type" => {
                    entity.media_type = MediaType::from(row.get::<String, &str>(column.name()))
                }
                "path" => entity.path = row.get(column.name()),
                "metadata" => {
                    let value: String = row.get(column.name());
                    if let Ok(metadata) = serde_json::from_str(value.as_str()) {
                        entity.metadata = metadata;
                    }
                }

                _ => panic!("New field added to the media table: {:?}", column.name()),
            }
        }

        if entity.internal_id > 0 {
            Some(entity)
        } else {
            None
        }
    }
}
