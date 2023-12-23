use crate::{entity::search::search_event_handler, web_app::web_app_event_handler};

pub(crate) async fn register_handlers() {
    let mut builder = orsomafo::EventDispatcherBuilder::new();

    builder = search_event_handler::register_handlers(builder);
    builder = web_app_event_handler::register_handlers(builder);

    builder.build().await;
}
