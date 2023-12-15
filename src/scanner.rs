use async_recursion::async_recursion;
use std::{fs::File, path::PathBuf};
use symphonia::core::{
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::{Hint, ProbeResult},
};
use tokio::fs::DirEntry;

use crate::{
    db::DbManager,
    entity::media::{InMediaEntityDto, MediaMetadata, MediaType},
};

pub(crate) async fn scan(path: String, db_manager: &DbManager) {
    println!("we are about to scan this path: {:?}", path);
    walk_dir(path.into(), db_manager).await
}

#[async_recursion(?Send)]
async fn walk_dir(path: PathBuf, db_manager: &DbManager) {
    match tokio::fs::read_dir(path).await {
        Ok(mut entries) => {
            while let Ok(Some(an_entry)) = entries.next_entry().await {
                let metadata = an_entry.metadata().await.unwrap();
                if metadata.is_dir() {
                    println!("read directory: {:?} ", an_entry.path());
                    walk_dir(an_entry.path(), db_manager).await
                } else {
                    process_entry(an_entry, db_manager).await;
                }
            }
        }
        Err(e) => println!("could not read director: {:?}", e),
    }
}

async fn process_entry(entry: DirEntry, db_manager: &DbManager) {
    let exts = ["mp3"];

    if let Some(ext) = entry.path().extension() {
        if exts.contains(&ext.to_str().unwrap()) {
            let mut hint = Hint::new();
            let path = entry.path();

            // Provide the file extension as a hint.
            if let Some(extension) = path.extension() {
                if let Some(extension_str) = extension.to_str() {
                    hint.with_extension(extension_str);
                }
            }

            let source = Box::new(File::open(path).unwrap()); // TODO: Handle the error
            let mss = MediaSourceStream::new(source, Default::default());

            // Use the default options for format readers other than for gapless playback.
            let format_opts = FormatOptions {
                enable_gapless: false,
                ..Default::default()
            };

            // Use the default options for metadata readers.
            let metadata_opts: MetadataOptions = Default::default();
            let mut media_metadata = MediaMetadata::default();

            if let Ok(probe_result) =
                symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)
            {
                println!("processing file: {:?}", entry.file_name());
                if let Some(m) = process_probe_result(probe_result).await {
                    media_metadata = m;
                }
            }

            _ = db_manager
                .media_repo()
                .create_or_update(InMediaEntityDto::new(
                    entry.file_name().to_str().unwrap(),
                    Some(match ext.to_str().unwrap() {
                        "mp3" | "acc" | "m4a" | "flacc" | "wav" => MediaType::Audio,
                        "mp4" | "avi" => MediaType::Video,
                        "jpg" | "png" | "gif" => MediaType::Photo,
                        _ => MediaType::default(),
                    }),
                    Some(entry.path().to_str().unwrap().to_owned()),
                    Some(media_metadata),
                ))
                .await;
        }
    }
}

async fn process_probe_result(mut probed: ProbeResult) -> Option<MediaMetadata> {
    if let Some(metadata_rev) = probed.format.metadata().current() {
        Some(MediaMetadata::from(metadata_rev.tags()))
    } else if let Some(metadata_rev) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
        Some(MediaMetadata::from(metadata_rev.tags()))
    } else {
        None
    }
}
