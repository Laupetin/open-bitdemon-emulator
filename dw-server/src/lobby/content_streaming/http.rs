use crate::lobby::content_streaming::publisher_file::DwPublisherContentStreamingService;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::response::FileStream;
use bitdemon::domain::title::Title;
use log::info;
use num_traits::FromPrimitive;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

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
) -> Result<Response, (StatusCode, String)> {
    info!("Streaming publisher file for {title_num} and {file_id}");

    let title = Title::from_u32(title_num)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Illegal title num".to_string()))?;

    let stream = publisher_service
        .stream_by_id(title, file_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Stream not found".to_string()))?;

    let file_name = format!("stream/publisher/{title_num}/{}", stream.filename);
    let file = File::open(file_name.as_str())
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("File not found: {e}")))?;

    let stream = ReaderStream::new(file);
    let file_stream_resp = FileStream::new(stream).file_name(file_name);

    Ok(file_stream_resp.into_response())
}
