use bitdemon::domain::result_slice::ResultSlice;
use bitdemon::lobby::service::storage::{
    FileVisibility, PublisherStorageService, StorageFileInfo, StorageHandler, StorageService,
    StorageServiceError,
};
use bitdemon::lobby::ThreadSafeLobbyHandler;
use bitdemon::networking::bd_session::BdSession;
use std::sync::Arc;

pub fn create_storage_handler() -> Arc<ThreadSafeLobbyHandler> {
    Arc::new(StorageHandler::new(
        Arc::new(DwStorageService::new()),
        Arc::new(DwPublisherStorageService::new()),
    ))
}

struct DwStorageService {}

impl StorageService for DwStorageService {
    fn get_storage_file_data_by_id(
        &self,
        session: &BdSession,
        owner_id: u64,
        file_id: u64,
    ) -> Result<Vec<u8>, StorageServiceError> {
        todo!()
    }

    fn get_storage_file_data_by_name(
        &self,
        session: &BdSession,
        owner_id: u64,
        filename: String,
    ) -> Result<Vec<u8>, StorageServiceError> {
        todo!()
    }

    fn list_storage_files(
        &self,
        session: &BdSession,
        owner_id: u64,
        min_date_time: i64,
        page_offset: usize,
        page_size: usize,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        todo!()
    }

    fn filter_storage_files(
        &self,
        session: &BdSession,
        owner_id: u64,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
        filter: String,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        todo!()
    }

    fn create_storage_file(
        &self,
        session: &BdSession,
        owner_id: u64,
        filename: String,
        visibility: FileVisibility,
        file_data: Vec<u8>,
    ) -> Result<StorageFileInfo, StorageServiceError> {
        todo!()
    }

    fn update_storage_file_data(
        &self,
        session: &BdSession,
        owner_id: u64,
        file_id: u64,
        file_data: Vec<u8>,
    ) -> Result<(), StorageServiceError> {
        todo!()
    }

    fn remove_storage_file(
        &self,
        session: &BdSession,
        owner_id: u64,
        filename: String,
    ) -> Result<(), StorageServiceError> {
        todo!()
    }
}

impl DwStorageService {
    pub fn new() -> DwStorageService {
        DwStorageService {}
    }
}

struct DwPublisherStorageService {}

impl PublisherStorageService for DwPublisherStorageService {
    fn get_publisher_file_data(
        &self,
        session: &BdSession,
        filename: String,
    ) -> Result<Vec<u8>, StorageServiceError> {
        todo!()
    }

    fn list_publisher_files(
        &self,
        session: &BdSession,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        todo!()
    }

    fn filter_publisher_files(
        &self,
        session: &BdSession,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
        filter: String,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        todo!()
    }
}

impl DwPublisherStorageService {
    fn new() -> DwPublisherStorageService {
        DwPublisherStorageService {}
    }
}
