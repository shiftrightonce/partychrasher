use crate::{
    db::DbManager,
    entity::{
        album::InAlbumEntityDto,
        album_artist::InAlbumArtistEntityDto,
        album_track::InAlbumTrackEntityDto,
        artist::InArtistEntityDto,
        artist_track::InArtistTrackEntityDto,
        client::InClientEntityDto,
        playlist::InPlaylistEntityDto,
        playlist_tracks::InPlaylistTrackEntityDto,
        track::{InTrackEntityDto, TrackMetadata},
    },
};

use rand::seq::SliceRandom;
use rand::Rng;

async fn seed_clients(db_manager: &DbManager, total: u64) {
    let client_repo = db_manager.client_repo();
    for _ in 0..total {
        client_repo
            .create(if rand::random() {
                InClientEntityDto {
                    name: None,
                    role: Some(crate::entity::Role::Admin),
                }
            } else {
                InClientEntityDto::default()
            })
            .await;
    }
}

async fn seed_artists(db_manager: &DbManager, total: u64) {
    let artist_repo = db_manager.artist_repo();

    for i in 0..total {
        let entity = InArtistEntityDto {
            name: format!("Artist {}", i + 1),
            metadata: None,
        };
        artist_repo.create(entity).await;
    }
}

async fn seed_tracks(db_manager: &DbManager, total: u64) {
    let track_repo = db_manager.track_repo();
    let artist_track_repo = db_manager.artist_track_repo();
    let artist_repo = db_manager.artist_repo();
    let mut rng = rand::thread_rng();
    let paths = [
        "./music/allthat.mp3",
        "./music/creativeminds.mp3",
        "./music/dreams.mp3",
        "./music/hey.mp3",
        "./music/lib1.mp3",
        "./music/lib2.mp3",
    ];

    for i in 0..total {
        let title = format!("track {}", i + 1);
        let path = Some(paths.choose(&mut rng).unwrap().to_string());
        let mut metadata = TrackMetadata::default();

        metadata.title = title.clone();

        let track = InTrackEntityDto::new(&title, path, Some(metadata));
        if let Some(track) = track_repo.create(track).await {
            for artist in artist_repo.select_random(rng.gen_range(1..=5)).await {
                _ = artist_track_repo
                    .create(InArtistTrackEntityDto {
                        artist_id: artist.id,
                        track_id: track.id.clone(),
                        is_feature: rand::random(),
                        metadata: None,
                    })
                    .await;
            }
        }
    }
}

async fn seed_albums(db_manager: &DbManager, total: u64) {
    let album_repo = db_manager.album_repo();
    let artist_repo = db_manager.artist_repo();
    let track_repo = db_manager.track_repo();
    let album_artist_repo = db_manager.album_artist_repo();
    let album_track_repo = db_manager.album_track_repo();

    let mut rng = rand::thread_rng();

    for i in 0..total {
        let title = format!("album {}", i + 1);
        if let Some(album) = album_repo
            .create(InAlbumEntityDto {
                title,
                metadata: None,
            })
            .await
        {
            for artist in artist_repo.select_random(1).await {
                let entity = InAlbumArtistEntityDto {
                    album_id: album.id.clone(),
                    artist_id: artist.id,
                    metadata: None,
                };
                _ = album_artist_repo.create(entity).await;
            }

            for track in track_repo.select_random(rng.gen_range(2..14)).await {
                let entity = InAlbumTrackEntityDto {
                    album_id: album.id.clone(),
                    track_id: track.id,
                    metadata: None,
                };
                _ = album_track_repo.create(entity).await;
            }
        }
    }
}

async fn seed_playlists(db_manager: &DbManager, total: u64) {
    let playlist_repo = db_manager.playlist_repo();
    let track_repo = db_manager.track_repo();
    let playlist_track_repo = db_manager.playlist_track_repo();

    let mut rng = rand::thread_rng();

    // default
    if let Some(default) = playlist_repo.get_default_playlist().await {
        for track in track_repo.select_random(rng.gen_range(2..100)).await {
            let in_entity = InPlaylistTrackEntityDto {
                playlist_id: default.id.clone(),
                track_id: track.id,
                metadata: None,
            };
            _ = playlist_track_repo.create(in_entity).await;
        }
    }

    for i in 0..total {
        let name = format!("Playlist {}", i + 1);
        let description = format!("Seeded playlist record {}", i + 1);
        let entity = InPlaylistEntityDto {
            name,
            description: Some(description),
            is_default: None,
        };

        if let Some(playlist) = playlist_repo.create(entity).await {
            for track in track_repo.select_random(rng.gen_range(2..10)).await {
                let in_entity = InPlaylistTrackEntityDto {
                    playlist_id: playlist.id.clone(),
                    track_id: track.id,
                    metadata: None,
                };
                _ = playlist_track_repo.create(in_entity).await;
            }
        }
    }
}

pub(crate) async fn run_seeders(db_manager: &DbManager, total: u64) {
    seed_clients(db_manager, total).await;
    seed_artists(db_manager, total).await;
    seed_tracks(db_manager, total).await;
    seed_albums(db_manager, total).await;
    seed_playlists(db_manager, total).await;
}
