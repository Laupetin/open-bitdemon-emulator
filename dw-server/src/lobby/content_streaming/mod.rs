use crate::lobby::content_streaming::http::create_content_streaming_router;
use crate::lobby::content_streaming::publisher_file::DwPublisherContentStreamingService;
use crate::lobby::content_streaming::user_file::DwUserContentStreamingService;
use crate::lobby::ConfiguredEnvironment;
use bitdemon::lobby::content_streaming::ContentStreamingHandler;
use bitdemon::lobby::LobbyServiceId;
use std::sync::Arc;

mod db;
mod http;
mod publisher_file;
mod user_file;

pub fn create_content_streaming_handler() -> ConfiguredEnvironment {
    let user_service = Arc::new(DwUserContentStreamingService::new());
    let publisher_service = Arc::new(DwPublisherContentStreamingService::new());

    let router = create_content_streaming_router(user_service.clone(), publisher_service.clone());

    ConfiguredEnvironment::new(
        LobbyServiceId::ContentStreaming,
        Arc::new(ContentStreamingHandler::new(
            user_service,
            publisher_service,
        )),
    )
    .with_pub_router(router)
}
