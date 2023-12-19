use std::{collections::HashMap, fmt::Display, str::FromStr, sync::Arc, time::Duration};

use actix_web::{web::Query, HttpRequest, HttpResponse};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};

use crate::{
    config::Config,
    entity::{
        album::AlbumRepo, album_artist::AlbumArtistRepo, album_track::AlbumTrackRepo,
        artist::ArtistRepo, artist_track::ArtistTrackRepo, client::ClientRepo, media::MediaRepo,
        playlist::PlaylistRepo, playlist_tracks::PlaylistTracksRepo, track::TrackRepo,
    },
    helper::{base64_decode_to_string, base64_encode},
};

pub(crate) type DbConnection = SqlitePool;

pub(crate) async fn setup_db_connection(config: &Config) -> Arc<DbManager> {
    let db = format!("{}/data.db", &config.db_path());
    let option = SqliteConnectOptions::from_str(&db)
        .unwrap()
        .foreign_keys(true)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(30));

    match SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(option)
        .await
    {
        Ok(pool) => Arc::new(DbManager::new(pool).await),
        Err(e) => {
            panic!("could not setup db: {:?}", e)
        }
    }
}

pub(crate) struct DbManager {
    pool: SqlitePool,
}

impl DbManager {
    async fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn client_repo(&self) -> ClientRepo {
        ClientRepo::new(self.pool.clone())
    }

    pub(crate) fn track_repo(&self) -> TrackRepo {
        TrackRepo::new(self.pool.clone())
    }

    pub(crate) fn playlist_track_repo(&self) -> PlaylistTracksRepo {
        PlaylistTracksRepo::new(self.pool.clone())
    }

    pub(crate) fn artist_repo(&self) -> ArtistRepo {
        ArtistRepo::new(self.pool.clone())
    }

    pub(crate) fn playlist_repo(&self) -> PlaylistRepo {
        PlaylistRepo::new(self.pool.clone())
    }

    pub(crate) fn album_repo(&self) -> AlbumRepo {
        AlbumRepo::new(self.pool.clone())
    }

    pub(crate) fn album_artist_repo(&self) -> AlbumArtistRepo {
        AlbumArtistRepo::new(self.pool.clone())
    }

    pub(crate) fn album_track_repo(&self) -> AlbumTrackRepo {
        AlbumTrackRepo::new(self.pool.clone())
    }

    pub(crate) fn artist_track_repo(&self) -> ArtistTrackRepo {
        ArtistTrackRepo::new(self.pool.clone())
    }

    pub(crate) fn media_repo(&self) -> MediaRepo {
        MediaRepo::new(self.pool.clone())
    }

    pub(crate) async fn setup_db(&self) {
        // clients table
        if self.client_repo().setup_table().await {
            if !self.client_repo().has_admin().await {
                if let Some(client) = self.client_repo().create_default_admin().await {
                    println!("Admin API Token: {}", &client.api_token());
                }

                if let Some(user) = self.client_repo().create_default_client().await {
                    println!("User API Token: {}", &user.api_token());
                }
            }
        }

        // artists table
        self.artist_repo().setup_table().await;

        // albums table
        self.album_repo().setup_table().await;
        // tracks table
        self.track_repo().setup_table().await;

        // album artists
        self.album_artist_repo().setup_table().await;

        // album tracks
        self.album_track_repo().setup_table().await;

        // playlists table
        if let Some(playlist) = self.playlist_repo().setup_table().await {
            println!("Default Playlist ID: {}", &playlist.id);
        }

        // playlist tracks table
        self.playlist_track_repo().setup_table().await;

        // artist tracks table
        self.artist_track_repo().setup_table().await;

        // media table
        self.media_repo().setup_table().await;
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PaginatorDirection {
    Next,
    Previous,
}

impl Default for PaginatorDirection {
    fn default() -> Self {
        Self::Next
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Paginator {
    pub(crate) current: u64,
    pub(crate) next: u64,
    pub(crate) previous: u64,
    pub(crate) limit: u64,
    pub(crate) last_value: String,
    pub(crate) direction: PaginatorDirection,
    pub(crate) order_field: String,
}

impl Default for Paginator {
    fn default() -> Self {
        Self {
            current: 0,
            next: 1,
            previous: 0,
            limit: 250,
            last_value: "".to_string(),
            direction: PaginatorDirection::Next,
            order_field: "id".to_string(),
        }
    }
}

impl Display for Paginator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let direction = match self.direction {
            PaginatorDirection::Next => "n",
            PaginatorDirection::Previous => "p",
        };
        let string = format!(
            "{}.{}.{}.{}.{}.{}.{}",
            self.current,
            self.next,
            self.previous,
            direction,
            self.order_field,
            self.limit,
            self.last_value
        );

        write!(f, "{}", base64_encode(&string))
    }
}

impl Paginator {
    pub(crate) fn next_from_current(current: &Self) -> Self {
        Self {
            current: current.next,
            next: current.next + 1,
            limit: current.limit,
            last_value: current.last_value.clone(),
            previous: if current.next > 0 {
                current.next - 1
            } else {
                0
            },
            order_field: current.order_field.clone(),
            direction: PaginatorDirection::Next,
        }
    }

    pub(crate) fn previous_from_current(current: &Self) -> Self {
        Self {
            current: current.previous,
            next: current.next,
            limit: current.limit,
            last_value: current.last_value.clone(),
            previous: if current.previous > 0 {
                current.previous - 1
            } else {
                0
            },
            order_field: current.order_field.clone(),
            direction: PaginatorDirection::Previous,
        }
    }

    pub(crate) fn to_collection(&self) -> HashMap<String, String> {
        let mut collection = HashMap::new();

        collection.insert("current".to_string(), self.to_string());
        collection.insert(
            "next".to_string(),
            Self::next_from_current(self).to_string(),
        );
        collection.insert(
            "previous".to_string(),
            Self::previous_from_current(self).to_string(),
        );

        collection
    }
}

impl From<String> for Paginator {
    fn from(value: String) -> Self {
        let mut pieces = value.split('.');
        let mut default = Self::default();

        // current
        if let Some(current) = pieces.next() {
            default.current = current.parse().unwrap_or(default.current);
        }

        // next
        if let Some(next) = pieces.next() {
            default.next = next.parse().unwrap_or(default.next);
        }

        // previous
        if let Some(previous) = pieces.next() {
            default.previous = previous.parse().unwrap_or(default.previous);
        }

        // direction
        if let Some(direction) = pieces.next() {
            default.direction = match direction {
                "n" => PaginatorDirection::Next,
                "p" => PaginatorDirection::Previous,
                _ => PaginatorDirection::default(),
            };
        }

        // order_field
        if let Some(order_field) = pieces.next() {
            default.order_field = order_field.to_string();
        }

        // limit
        if let Some(limit) = pieces.next() {
            default.limit = limit.parse().unwrap_or(default.limit)
        }

        // last value
        if let Some(last_value) = pieces.next() {
            default.last_value = last_value.to_string()
        }

        default
    }
}

impl TryFrom<&HttpRequest> for Paginator {
    type Error = String;

    fn try_from(value: &HttpRequest) -> Result<Self, Self::Error> {
        let query = Query::<HashMap<String, String>>::from_query(value.query_string()).unwrap();
        if let Some(page) = query.get("page") {
            Ok(if let Some(the_string) = base64_decode_to_string(page) {
                Paginator::from(the_string)
            } else {
                Paginator::default()
            })
        } else {
            // TODO: Source the paginator pieces if they exist for example
            //       page_index, page_field, page_limit, page_dir,
            Ok(Paginator::default())
        }
    }
}

#[derive(serde::Serialize)]
pub(crate) struct PaginatedResult<T: serde::Serialize> {
    page: T,
    paginators: HashMap<String, String>,
}

impl<T: serde::Serialize> PaginatedResult<T> {
    pub(crate) fn new(page: T, paginator: &Paginator) -> Self {
        Self {
            page,
            paginators: paginator.to_collection(),
        }
    }

    pub(crate) fn into_response(self) -> HttpResponse {
        HttpResponse::Ok().json(self)
    }
}
