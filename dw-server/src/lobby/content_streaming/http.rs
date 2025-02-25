use crate::lobby::content_streaming::publisher_file::DwPublisherContentStreamingService;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use log::info;
use std::sync::Arc;

pub fn create_content_streaming_router(
    publisher_service: Arc<DwPublisherContentStreamingService>,
) -> Router {
    Router::new()
        .route(
            "/content/publisher/{title}/{file_id}",
            get(retrieve_publisher_file),
        )
        .with_state(publisher_service)
}

async fn retrieve_publisher_file(
    Path((title_num, file_id)): Path<(u32, u64)>,
    State(publisher_service): State<Arc<DwPublisherContentStreamingService>>,
) {
    info!("Retrieving publisher file for {title_num} and {file_id}")
}
