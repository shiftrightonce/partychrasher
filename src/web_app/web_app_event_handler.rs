use orsomafo::EventDispatcherBuilder;

mod web_app_playlist_track_event_handler;

pub(crate) fn register_handlers(mut builder: EventDispatcherBuilder) -> EventDispatcherBuilder {
    builder = web_app_playlist_track_event_handler::register(builder);

    builder
}
