use crate::lobby::content_streaming::publisher_file::DwPublisherContentStreamingService;
use crate::lobby::content_streaming::user_file::DwUserContentStreamingService;
use bitdemon::lobby::content_streaming::ContentStreamingHandler;
use bitdemon::lobby::ThreadSafeLobbyHandler;
use std::sync::Arc;

mod db;
mod publisher_file;
mod user_file;

pub fn create_content_streaming_handler() -> Arc<ThreadSafeLobbyHandler> {
    Arc::new(ContentStreamingHandler::new(
        Arc::new(DwUserContentStreamingService::new()),
        Arc::new(DwPublisherContentStreamingService::new()),
    ))
}
