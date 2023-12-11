use std::collections::HashMap;

use actix_web::{
    error::{ErrorNotFound, ErrorUnauthorized},
    web, HttpRequest,
};

use crate::entity::client::OutClientEntityDto;

use super::when_admin;

pub(crate) async fn serve(
    id: web::Path<String>,
    req: HttpRequest,
) -> actix_web::Result<actix_files::NamedFile> {
    let (_, response) = when_admin::<OutClientEntityDto>(&req).await;

    if response.is_some() {
        return Err(ErrorUnauthorized("Permission denied"));
    }

    // dummy files
    let mut dummy = HashMap::<&str, &str>::new();
    dummy.insert("track1", "./music/hey.mp3");
    dummy.insert("track2", "./music/allthat.mp3");
    dummy.insert("track3", "./music/creativeminds.mp3");
    dummy.insert("track4", "./music/dreams.mp3");
    dummy.insert("track5", "./music/lib1.mp3");
    dummy.insert("track6", "./music/lib2.mp3");
    dummy.insert("video1", "./music/video1.mp4");
    dummy.insert("video2", "./music/video2.mp4");
    dummy.insert("video3", "./music/video3.mp4");

    let the_id = id.into_inner();
    if let Some(file) = dummy.get(the_id.as_str()) {
        actix_files::NamedFile::open(file).map_err(Into::into)
    } else {
        return Err(ErrorNotFound(format!(
            "file with ID: {:?} not found",
            the_id
        )));
    }
}
