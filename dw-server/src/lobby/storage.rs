use bitdemon::domain::result_slice::ResultSlice;
use bitdemon::domain::title::Title;
use bitdemon::lobby::service::storage::{
    FileVisibility, PublisherStorageService, StorageFileInfo, StorageHandler, StorageService,
    StorageServiceError,
};
use bitdemon::lobby::ThreadSafeLobbyHandler;
use bitdemon::networking::bd_session::BdSession;
use num_traits::ToPrimitive;
use std::fs;
use std::fs::DirEntry;
use std::os::windows::fs::MetadataExt;
use std::path::{Component, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

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
        let path_buf = PathBuf::from_str(&filename)
            .or_else(|_| Err(StorageServiceError::StorageFileNotFoundError))?;

        let directory_traversal = path_buf
            .components()
            .into_iter()
            .any(|component| component == Component::ParentDir);

        if directory_traversal {
            return Err(StorageServiceError::StorageFileNotFoundError);
        }

        let full_file_path = format!(
            "./storage/publisher/{}/{filename}",
            session.authentication().unwrap().title.to_u32().unwrap()
        );

        let buf = fs::read(full_file_path)
            .or_else(|_| Err(StorageServiceError::StorageFileNotFoundError))?;

        Ok(buf)
    }

    fn list_publisher_files(
        &self,
        session: &BdSession,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        let title = session.authentication().unwrap().title;
        let full_dir_path = format!("./storage/publisher/{}", title.to_u32().unwrap());

        let dir = fs::read_dir(full_dir_path);
        if dir.is_err() {
            return Ok(ResultSlice::new(Vec::new(), item_offset));
        }

        let file_info = dir
            .unwrap()
            .filter(|entry| entry.is_ok())
            .skip(item_offset)
            .map(|entry| entry.unwrap())
            .map(|entry| Self::map_info_info(title, entry))
            .filter(|info| info.created >= min_date_time)
            .take(item_count)
            .collect();

        Ok(ResultSlice::new(file_info, item_offset))
    }

    fn filter_publisher_files(
        &self,
        session: &BdSession,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
        filter: String,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        let title = session.authentication().unwrap().title;
        let full_dir_path = format!("./storage/publisher/{}", title.to_u32().unwrap());

        let dir = fs::read_dir(full_dir_path);
        if dir.is_err() {
            return Ok(ResultSlice::new(Vec::new(), item_offset));
        }

        let file_info = dir
            .unwrap()
            .filter(|entry| entry.is_ok())
            .filter(|entry| {
                entry
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    .starts_with(&filter)
            })
            .skip(item_offset)
            .map(|entry| entry.unwrap())
            .map(|entry| Self::map_info_info(title, entry))
            .filter(|info| info.created >= min_date_time)
            .take(item_count)
            .collect();

        Ok(ResultSlice::new(file_info, item_offset))
    }
}

impl DwPublisherStorageService {
    fn new() -> DwPublisherStorageService {
        DwPublisherStorageService {}
    }

    fn map_info_info(title: Title, entry: DirEntry) -> StorageFileInfo {
        let metadata = entry.metadata().unwrap();
        StorageFileInfo {
            id: 0,
            filename: entry.file_name().into_string().unwrap(),
            title,
            file_size: metadata.file_size(),
            created: metadata
                .created()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            modified: metadata
                .modified()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            visibility: FileVisibility::VisiblePublic,
            owner_id: 0,
        }
    }
}
