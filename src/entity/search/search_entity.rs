use serde_json::Map;
use sqlx::{Column, Row};

use crate::entity::{
    album::AlbumEntity, artist::ArtistEntity, playlist::PlaylistEntity, track::TrackEntity,
    FromSqliteRow,
};

pub(crate) struct SearchHitEntity {
    internal_id: i64,
    id: String,
    entity: String,
    entity_id: String,
    metadata: Map<String, serde_json::Value>,
}

#[derive(Debug, Default)]
pub(crate) struct InSearchHitEntityDto {
    pub(crate) keywords: Vec<String>,
    pub(crate) entity: String,
    pub(crate) entity_id: String,
    pub(crate) metadata: Map<String, serde_json::Value>,
}

impl InSearchHitEntityDto {
    pub(crate) fn metadata_to_string(&self) -> String {
        serde_json::to_string(&self.metadata).unwrap()
    }
}

impl Default for SearchHitEntity {
    fn default() -> Self {
        Self {
            internal_id: 0,
            id: String::new(),
            entity: String::new(),
            entity_id: String::new(),
            metadata: Map::new(),
        }
    }
}

impl FromSqliteRow for SearchHitEntity {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> Option<Self>
    where
        Self: Sized,
    {
        let mut entity = Self::default();
        for column in row.columns() {
            match column.name() {
                "internal_id" => entity.internal_id = row.get(column.name()),
                "id" => entity.id = row.get(column.name()),
                "entity" => entity.entity = row.get(column.name()),
                "entity_id" => entity.entity_id = row.get(column.name()),
                "metadata" => {
                    let value: String = row.get(column.name());
                    if let Ok(json) = serde_json::from_str(&value) {
                        entity.metadata = json;
                    }
                }
                _ => panic!(
                    "New field added to the search hits table: {}",
                    column.name()
                ),
            }
        }

        if entity.internal_id > 0 {
            Some(entity)
        } else {
            None
        }
    }
}

#[derive(Debug, Default, serde::Serialize)]
pub(crate) struct OutSearchHitEntityDto {
    id: String,
    entity: String,
    entity_id: String,
    metadata: Map<String, serde_json::Value>,
}

impl From<SearchHitEntity> for OutSearchHitEntityDto {
    fn from(entity: SearchHitEntity) -> Self {
        Self {
            id: entity.id,
            entity: entity.entity,
            entity_id: entity.entity_id,
            metadata: entity.metadata,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct SearchEntity {
    pub(crate) internal_id: i64,
    pub(crate) term: String,
}

impl FromSqliteRow for SearchEntity {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> Option<Self>
    where
        Self: Sized,
    {
        let mut search = Self::default();
        for column in row.columns() {
            match column.name() {
                "internal_id" => search.internal_id = row.get(column.name()),
                "term" => search.term = row.get(column.name()),
                _ => (),
            }
        }

        if search.term.is_empty() {
            None
        } else {
            Some(search)
        }
    }
}

impl From<&TrackEntity> for InSearchHitEntityDto {
    fn from(track: &TrackEntity) -> Self {
        let mut keywords = vec![track.title.clone()];

        if !track.metadata.genre.is_empty() {
            keywords.push(track.metadata.genre.clone());
        }

        let mut metadata = Map::new();

        metadata.insert("title".to_string(), track.title.clone().into());
        metadata.insert(
            "entity_metadata".to_string(),
            serde_json::to_value(track.metadata.clone()).unwrap(),
        );
        Self {
            keywords,
            entity: "track".to_string(),
            entity_id: track.id.clone(),
            metadata,
        }
    }
}

impl From<TrackEntity> for InSearchHitEntityDto {
    fn from(value: TrackEntity) -> Self {
        Self::from(&value)
    }
}

impl From<&AlbumEntity> for InSearchHitEntityDto {
    fn from(album: &AlbumEntity) -> Self {
        let keywords = vec![album.title.clone()];

        let mut metadata = Map::new();
        metadata.insert("title".to_string(), album.title.clone().into());
        metadata.insert(
            "entity_metadata".to_string(),
            serde_json::to_value(album.metadata.clone()).unwrap(),
        );
        Self {
            keywords,
            entity: "album".to_string(),
            entity_id: album.id.clone(),
            metadata,
        }
    }
}

impl From<AlbumEntity> for InSearchHitEntityDto {
    fn from(value: AlbumEntity) -> Self {
        Self::from(&value)
    }
}

impl From<&ArtistEntity> for InSearchHitEntityDto {
    fn from(artist: &ArtistEntity) -> Self {
        let keywords = vec![artist.name.clone()];

        let mut metadata = Map::new();
        metadata.insert("name".to_string(), artist.name.clone().into());
        metadata.insert(
            "entity_metadata".to_string(),
            artist.metadata.clone().into(),
        );
        Self {
            keywords,
            entity: "artist".to_string(),
            entity_id: artist.id.clone(),
            metadata,
        }
    }
}

impl From<&PlaylistEntity> for InSearchHitEntityDto {
    fn from(playlist: &PlaylistEntity) -> Self {
        let keywords = vec![playlist.name.clone()];
        let mut metadata = Map::new();
        metadata.insert("name".to_string(), playlist.name.clone().into());
        let mut playlist_metadata = serde_json::Map::new();
        playlist_metadata.insert(
            "description".to_string(),
            playlist.description.clone().into(),
        );
        metadata.insert("entity_metadata".to_string(), playlist_metadata.into());
        Self {
            keywords,
            entity: "playlist".to_string(),
            entity_id: playlist.id.clone(),
            metadata,
        }
    }
}

impl From<PlaylistEntity> for InSearchHitEntityDto {
    fn from(value: PlaylistEntity) -> Self {
        Self::from(&value)
    }
}
