use crate::lobby::content_streaming::publisher_file::DwPublisherContentStreamingService;
use crate::lobby::content_streaming::user_file::{
    DwUserContentStreamingService, UserFileClaimOperation, UserFileClaims,
};
use axum::body::{Body, Bytes};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::response::FileStream;
use bitdemon::domain::title::Title;
use jsonwebtoken::{decode, Validation};
use log::info;
use num_traits::FromPrimitive;
use serde::Deserialize;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[derive(Deserialize)]
struct UserStreamQuery {
    authorization: String,
}

pub fn create_content_streaming_router(
    user_service: Arc<DwUserContentStreamingService>,
    publisher_service: Arc<DwPublisherContentStreamingService>,
) -> Router {
    let publisher_router = Router::new()
        .route("/{title}/{stream_id}", get(retrieve_publisher_file))
        .with_state(publisher_service);

    let user_router: Router = Router::new()
        .route(
            "/{title}/{stream_id}",
            get(retrieve_user_file)
                .put(upload_user_file)
                .delete(delete_user_file),
        )
        .with_state(user_service);

    Router::new()
        .nest("/content/publisher", publisher_router)
        .nest("/content/user", user_router)
}

async fn retrieve_publisher_file(
    Path((title_num, stream_id)): Path<(u32, u64)>,
    State(publisher_service): State<Arc<DwPublisherContentStreamingService>>,
) -> Result<Response, (StatusCode, String)> {
    info!("Streaming publisher file for {title_num} and {stream_id}");

    let title = Title::from_u32(title_num)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Illegal title num".to_string()))?;

    let stream = publisher_service
        .stream_by_id(title, stream_id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Stream not found".to_string()))?;

    let file_name = format!("stream/publisher/{title_num}/{}", stream.filename);
    let file = File::open(file_name.as_str())
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("File not found: {e}")))?;

    let stream = ReaderStream::new(file);
    let file_stream_resp = FileStream::new(stream).file_name(file_name);

    Ok(file_stream_resp.into_response())
}

async fn retrieve_user_file(
    State(user_service): State<Arc<DwUserContentStreamingService>>,
    Query(user_stream_query): Query<UserStreamQuery>,
    Path((title_num, stream_id)): Path<(u32, u64)>,
) -> Result<Response, StatusCode> {
    info!("Streaming user file for {title_num} and {stream_id}");

    validate_jwt(
        user_stream_query,
        title_num,
        stream_id,
        UserFileClaimOperation::Stream,
        user_service.as_ref(),
    )?;

    let title = Title::from_u32(title_num).ok_or(StatusCode::BAD_REQUEST)?;

    let stream = user_service
        .stream_by_id(title, stream_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Response::new(Body::from(stream)))
}

async fn upload_user_file(
    State(user_service): State<Arc<DwUserContentStreamingService>>,
    Query(user_stream_query): Query<UserStreamQuery>,
    Path((title_num, stream_id)): Path<(u32, u64)>,
    body: Bytes,
) -> Result<(), StatusCode> {
    info!("Uploading user stream for {title_num} and {stream_id}");

    validate_jwt(
        user_stream_query,
        title_num,
        stream_id,
        UserFileClaimOperation::Create,
        user_service.as_ref(),
    )?;

    let title = Title::from_u32(title_num).ok_or(StatusCode::BAD_REQUEST)?;

    let data = body.to_vec();

    if user_service.set_stream_data(title, stream_id, data) {
        Ok(())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn delete_user_file(
    State(user_service): State<Arc<DwUserContentStreamingService>>,
    Query(user_stream_query): Query<UserStreamQuery>,
    Path((title_num, stream_id)): Path<(u32, u64)>,
) -> Result<(), StatusCode> {
    info!("Deleting user stream for {title_num} and {stream_id}");

    validate_jwt(
        user_stream_query,
        title_num,
        stream_id,
        UserFileClaimOperation::Delete,
        user_service.as_ref(),
    )?;

    let title = Title::from_u32(title_num).ok_or(StatusCode::BAD_REQUEST)?;

    if user_service.delete_stream(title, stream_id) {
        Ok(())
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

fn validate_jwt(
    query: UserStreamQuery,
    title_num: u32,
    stream_id: u64,
    operation: UserFileClaimOperation,
    user_service: &DwUserContentStreamingService,
) -> Result<(), StatusCode> {
    let jwt = decode::<UserFileClaims>(
        query.authorization.as_str(),
        &user_service.decoding_key,
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if jwt.claims.stream_title != title_num
        || jwt.claims.stream_id != stream_id
        || jwt.claims.stream_operation != operation
    {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(())
}
