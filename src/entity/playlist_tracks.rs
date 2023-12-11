mod playlist_tracks_repo;
mod playlist_tracts_entity;

pub use playlist_tracks_repo::*;
pub use playlist_tracts_entity::*;

/*
CREATE TABLE "playlist_tracts" (
    "track_id"	TEXT NOT NULL,
    "playlist_id"	TEXT NOT NULL,
    UNIQUE("track_id","playlist_id")
);
 */
