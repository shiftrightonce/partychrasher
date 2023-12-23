use orsomafo::EventDispatcherBuilder;

mod search_album_event_handler;
mod search_playlist_event_handler;
mod search_track_event_handler;

pub(crate) fn register_handlers(mut builder: EventDispatcherBuilder) -> EventDispatcherBuilder {
    builder = search_track_event_handler::register(builder);
    builder = search_album_event_handler::register(builder);
    builder = search_playlist_event_handler::register(builder);

    builder
}
