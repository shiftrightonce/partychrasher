use async_recursion::async_recursion;
use lofty::MimeType;
use std::path::PathBuf;
use tokio::fs::DirEntry;

use crate::{
    config::Config,
    db::DbManager,
    entity::{
        album::{AlbumMetadata, InAlbumEntityDto},
        album_artist::InAlbumArtistEntityDto,
        album_track::InAlbumTrackEntityDto,
        artist::{ArtistEntity, InArtistEntityDto},
        artist_track::InArtistTrackEntityDto,
        media::{InMediaEntityDto, MediaEntity, MediaMetadata, MediaType},
        track::TrackEntity,
    },
};

pub(crate) async fn scan(path: String, db_manager: &DbManager, config: &Config) {
    println!("we are about to scan this path: {:?}", path);
    walk_dir(path.into(), db_manager, config).await
}

#[async_recursion(?Send)]
async fn walk_dir(path: PathBuf, db_manager: &DbManager, config: &Config) {
    match tokio::fs::read_dir(path).await {
        Ok(mut entries) => {
            while let Ok(Some(an_entry)) = entries.next_entry().await {
                let metadata = an_entry.metadata().await.unwrap();
                if metadata.is_dir() {
                    println!("read directory: {:?} ", an_entry.path());
                    walk_dir(an_entry.path(), db_manager, config).await
                } else {
                    process_entry(an_entry, db_manager, config).await;
                }
            }
        }
        Err(e) => println!("could not read director: {:?}", e),
    }
}

async fn process_entry(entry: DirEntry, db_manager: &DbManager, config: &Config) {
    let exts = config.audio_format();

    if let Some(ext) = entry.path().extension() {
        if exts.contains(&ext.to_str().unwrap()) {
            let path = entry.path();
            println!("processing file: {:?}", entry.file_name());

            let media_metadata = lofty_tag_processor(&path, db_manager, config).await;

            if let Some(the_media) = db_manager
                .media_repo()
                .create_or_update(InMediaEntityDto::new_from_str(
                    entry.file_name().to_str().unwrap(),
                    ext.to_str().unwrap(),
                    Some(entry.path().to_str().unwrap().to_owned()),
                    Some(media_metadata),
                ))
                .await
            {
                if the_media.is_audio() {
                    let add_track_result = add_track(&the_media, db_manager).await;
                    if add_track_result.1.is_some() {
                        add_album(
                            &the_media,
                            add_track_result.0.as_ref().unwrap(),
                            add_track_result.1.as_ref().unwrap(),
                            db_manager,
                        )
                        .await;
                    }
                }
            }
        }
    }
}

async fn add_track(
    media: &MediaEntity,
    db_manager: &DbManager,
) -> (Option<TrackEntity>, Option<Vec<ArtistEntity>>) {
    let mut artists = Vec::new();
    let mut track = TrackEntity::default();
    if let Ok(in_track) = media.try_into() {
        if let Some(t) = db_manager.track_repo().create_or_update(in_track).await {
            track = t;
            if !track.metadata.artist.is_empty() {
                // create or update the artist record
                for artist_entry in track
                    .metadata
                    .artist
                    .split(',')
                    .map(|x| x.trim())
                    .collect::<Vec<&str>>()
                    .into_iter()
                    .enumerate()
                {
                    if let Some(artist) = db_manager
                        .artist_repo()
                        .create_or_update(InArtistEntityDto {
                            name: artist_entry.1.to_string(),
                            metadata: None,
                        })
                        .await
                    {
                        // assign this track to this artist
                        _ = db_manager
                            .artist_track_repo()
                            .create(InArtistTrackEntityDto {
                                artist_id: artist.id.clone(),
                                track_id: track.id.clone(),
                                is_feature: artist_entry.0 != 0,
                                metadata: None,
                            })
                            .await;
                        artists.push(artist)
                    }
                }
            }
        }
    }

    if track.internal_id > 0 {
        (Some(track), Some(artists))
    } else {
        (None, None)
    }
}

async fn add_album(
    media: &MediaEntity,
    track: &TrackEntity,
    artists: &[ArtistEntity],
    db_manager: &DbManager,
) {
    if !media.metadata.album.is_empty() {
        if let Some(album) = db_manager
            .album_repo()
            .create(InAlbumEntityDto {
                title: media.metadata.album.clone(),
                year: if media.metadata.year > 0 {
                    Some(media.metadata.year)
                } else {
                    None
                },
                metadata: Some(AlbumMetadata::from(&track.metadata)),
            })
            .await
        {
            // Add the track to this album
            _ = db_manager
                .album_track_repo()
                .create(InAlbumTrackEntityDto {
                    album_id: album.id.clone(),
                    track_id: track.id.clone(),
                    metadata: None,
                })
                .await;

            // Add artist to this album
            for artist_entry in artists.iter().enumerate() {
                if artist_entry.0 == 0 {
                    _ = db_manager
                        .album_artist_repo()
                        .create(InAlbumArtistEntityDto {
                            album_id: album.id.clone(),
                            artist_id: artist_entry.1.id.clone(),
                            metadata: None,
                        })
                        .await;
                }
            }
        }
    }
}

async fn lofty_tag_processor(
    path: &PathBuf,
    db_manager: &DbManager,
    config: &Config,
) -> MediaMetadata {
    use lofty::{Probe, TaggedFileExt};
    let mut metadata = MediaMetadata::default();

    if let Ok(reader) = Probe::open(path) {
        if let Ok(tagged_file) = reader.read() {
            let tag = match tagged_file.primary_tag() {
                Some(tag) => tag,
                None => tagged_file.first_tag().unwrap(),
            };

            metadata = MediaMetadata::from(tag);

            for a_picture in tag.pictures() {
                let extension = match a_picture.mime_type() {
                    MimeType::Png => ".png",
                    MimeType::Jpeg => ".jpg",
                    MimeType::Tiff => ".jpg",
                    MimeType::Bmp => ".bmp",
                    MimeType::Gif => ".gif",
                    MimeType::Unknown(t) => t.as_str(),
                    _ => "",
                };

                let pic_type = a_picture.pic_type();
                let media_type = pic_type.as_ape_key();
                if !extension.is_empty() && media_type.is_some() {
                    let dir = config.artwork_path();
                    let filename = format!("{}{}", sha256::digest(&metadata.album), extension);
                    let path = format!("{}/{}", dir, filename);

                    // Transforms `Cover Art (Other)` to `cover_art_other`
                    let pict_type_name = media_type
                        .unwrap()
                        .replace('(', "")
                        .replace(')', "")
                        .replace(' ', "_")
                        .to_lowercase();

                    if let Some(media) = db_manager
                        .media_repo()
                        .find_by_filename_and_path(&filename, &path)
                        .await
                    {
                        metadata.pictures.insert(pict_type_name, media.id);
                    } else if let Ok(_) = tokio::fs::write(&path, a_picture.data()).await {
                        if let Some(media) = db_manager
                            .media_repo()
                            .create_or_update(InMediaEntityDto {
                                filename,
                                media_type: Some(MediaType::Photo),
                                path: Some(path),
                                metadata: None,
                            })
                            .await
                        {
                            metadata.pictures.insert(pict_type_name, media.id);
                        }
                    };
                }
            }
        }
    }

    metadata
}
