use crate::lobby::storage::publisher_file::DwPublisherStorageService;
use crate::lobby::storage::user_file::DwUserStorageService;
use bitdemon::lobby::handler::storage::StorageHandler;
use bitdemon::lobby::ThreadSafeLobbyHandler;
use std::sync::Arc;

mod db;
mod publisher_file;
mod user_file;

pub fn create_storage_handler() -> Arc<ThreadSafeLobbyHandler> {
    Arc::new(StorageHandler::new(
        Arc::new(DwUserStorageService::new()),
        Arc::new(DwPublisherStorageService::new()),
    ))
}
