use crate::domain::result_slice::ResultSlice;
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::storage::result::FileDataResult;
use crate::lobby::storage::service::{
    FileVisibility, StorageFileInfo, StorageServiceError, ThreadSafePublisherStorageService,
    ThreadSafeUserStorageService,
};
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::warn;
use num_traits::FromPrimitive;
use std::error::Error;
use std::sync::Arc;

pub struct StorageHandler {
    storage_service: Arc<ThreadSafeUserStorageService>,
    publisher_storage_service: Arc<ThreadSafePublisherStorageService>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum StorageTaskId {
    // UploadFileAndDeleteMail
    // GetFilesByID
    UploadFile = 1,
    RemoveFile = 2,
    GetFile = 3,
    GetFileById = 4,
    ListFilesByOwner = 5,
    ListAllPublisherFiles = 6,
    GetPublisherFile = 7,
    UpdateFile = 8,

    // 9 = ?
    RemoveFile2 = 11,
    GetFile2 = 12,
    ListFilesByOwner2 = 13,
}

impl LobbyHandler for StorageHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = StorageTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return Ok(
                TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                    .to_response()?,
            );
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            StorageTaskId::UploadFile => self.upload_file(session, &mut message.reader),
            StorageTaskId::RemoveFile => self.remove_file(session, &mut message.reader),
            StorageTaskId::GetFile => self.get_file(session, &mut message.reader),
            StorageTaskId::GetFileById => self.get_file_by_id(session, &mut message.reader),
            StorageTaskId::ListFilesByOwner => {
                self.list_files_by_owner(session, &mut message.reader)
            }
            StorageTaskId::ListAllPublisherFiles => {
                self.list_all_publisher_files(session, &mut message.reader)
            }
            StorageTaskId::GetPublisherFile => {
                self.get_publisher_file(session, &mut message.reader)
            }
            StorageTaskId::UpdateFile => self.update_file(session, &mut message.reader),
            StorageTaskId::RemoveFile2
            | StorageTaskId::GetFile2
            | StorageTaskId::ListFilesByOwner2 => {
                warn!("Client called unimplemented task {task_id:?}");
                Ok(TaskReply::with_only_error_code(BdErrorCode::NoError, task_id).to_response()?)
            }
        }
    }
}

impl StorageHandler {
    pub fn new(
        storage_service: Arc<ThreadSafeUserStorageService>,
        publisher_storage_service: Arc<ThreadSafePublisherStorageService>,
    ) -> StorageHandler {
        StorageHandler {
            storage_service,
            publisher_storage_service,
        }
    }

    fn upload_file(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let filename = reader.read_str()?;
        let is_public = reader.read_bool()?;
        let file_data = reader.read_blob()?;

        let mut owner_id = session.authentication().unwrap().user_id;
        if reader.next_is_u64().unwrap_or(false) {
            owner_id = reader.read_u64()?;
        }

        let visibility = if is_public {
            FileVisibility::VisiblePublic
        } else {
            FileVisibility::VisiblePrivate
        };

        let result = self
            .storage_service
            .create_storage_file(session, owner_id, filename, visibility, file_data);

        match result {
            Ok(info) => Ok(TaskReply::with_results(
                StorageTaskId::UploadFile,
                vec![Box::from(info)],
            )
            .to_response()?),
            Err(error) => Ok(TaskReply::with_only_error_code(
                error.into(),
                StorageTaskId::UploadFile,
            )
            .to_response()?),
        }
    }

    fn remove_file(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let filename = reader.read_str()?;

        let mut owner_id = session.authentication().unwrap().user_id;
        if reader.next_is_u64().unwrap_or(false) {
            owner_id = reader.read_u64()?;
        }

        let result = self
            .storage_service
            .remove_storage_file(session, owner_id, filename);

        self.answer_for_no_return_value(StorageTaskId::RemoveFile, result)
    }

    fn get_file(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let filename = reader.read_str()?;
        let mut owner_id = reader.read_u64()?;

        if owner_id == 0 {
            owner_id = session.authentication().unwrap().user_id;
        }

        let result = self
            .storage_service
            .get_storage_file_data_by_name(session, owner_id, filename);

        self.answer_for_file_data(StorageTaskId::GetFile, result)
    }

    fn get_file_by_id(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let file_id = reader.read_u64()?;

        let result = self.storage_service.get_storage_file_data_by_id(
            session,
            session.authentication().unwrap().user_id,
            file_id,
        );

        self.answer_for_file_data(StorageTaskId::GetFileById, result)
    }

    fn list_files_by_owner(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let owner_id = reader.read_u64()?;
        let start_date = reader.read_u32()?;
        let max_num_results = reader.read_u16()?;
        let result_offset = reader.read_u16()?;

        let result;
        if reader.next_is_str().unwrap_or(false) {
            let filter = reader.read_str()?;
            result = self.storage_service.filter_storage_files(
                session,
                owner_id,
                start_date as i64,
                result_offset as usize,
                max_num_results as usize,
                filter,
            );
        } else {
            result = self.storage_service.list_storage_files(
                session,
                owner_id,
                start_date as i64,
                result_offset as usize,
                max_num_results as usize,
            );
        };

        self.answer_for_file_info_slice(StorageTaskId::ListFilesByOwner, result)
    }

    fn list_all_publisher_files(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let start_date = reader.read_u32()?;
        let max_num_results = reader.read_u16()?;
        let result_offset = reader.read_u16()?;

        let result;
        if reader.next_is_str().unwrap_or(false) {
            let filter = reader.read_str()?;
            result = self.publisher_storage_service.filter_publisher_files(
                session,
                start_date as i64,
                result_offset as usize,
                max_num_results as usize,
                filter,
            );
        } else {
            result = self.publisher_storage_service.list_publisher_files(
                session,
                start_date as i64,
                result_offset as usize,
                max_num_results as usize,
            );
        };

        self.answer_for_file_info_slice(StorageTaskId::ListAllPublisherFiles, result)
    }

    fn get_publisher_file(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let filename = reader.read_str()?;

        let result = self
            .publisher_storage_service
            .get_publisher_file_data(session, filename.clone());

        self.answer_for_file_data(StorageTaskId::GetPublisherFile, result)
    }

    fn update_file(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let file_id = reader.read_u64()?;
        let file_data = reader.read_blob()?;

        let result = self.storage_service.update_storage_file_data(
            session,
            session.authentication().unwrap().user_id,
            file_id,
            file_data,
        );

        self.answer_for_no_return_value(StorageTaskId::UpdateFile, result)
    }

    fn answer_for_file_data(
        &self,
        task_id: StorageTaskId,
        result: Result<Vec<u8>, StorageServiceError>,
    ) -> Result<BdResponse, Box<dyn Error>> {
        match result {
            Ok(data) => Ok(TaskReply::with_results(
                task_id,
                vec![Box::from(FileDataResult { data })],
            )
            .to_response()?),
            Err(error) => Ok(TaskReply::with_only_error_code(error.into(), task_id).to_response()?),
        }
    }

    fn answer_for_file_info_slice(
        &self,
        task_id: StorageTaskId,
        result: Result<ResultSlice<StorageFileInfo>, StorageServiceError>,
    ) -> Result<BdResponse, Box<dyn Error>> {
        match result {
            Ok(info) => {
                Ok(TaskReply::with_result_slice(task_id, info.serializable()).to_response()?)
            }
            Err(error) => Ok(TaskReply::with_only_error_code(error.into(), task_id).to_response()?),
        }
    }

    fn answer_for_no_return_value(
        &self,
        task_id: StorageTaskId,
        result: Result<(), StorageServiceError>,
    ) -> Result<BdResponse, Box<dyn Error>> {
        match result {
            Ok(_) => {
                Ok(TaskReply::with_only_error_code(BdErrorCode::NoError, task_id).to_response()?)
            }
            Err(error) => Ok(TaskReply::with_only_error_code(error.into(), task_id).to_response()?),
        }
    }
}

impl Into<BdErrorCode> for StorageServiceError {
    fn into(self) -> BdErrorCode {
        match self {
            StorageServiceError::PermissionDeniedError => BdErrorCode::PermissionDenied,
            StorageServiceError::FilenameTooLongError => BdErrorCode::FilenameMaxLengthExceeded,
            StorageServiceError::StorageFileTooLargeError => BdErrorCode::FileSizeLimitExceeded,
            StorageServiceError::StorageFileNotFoundError => BdErrorCode::NoFile,
        }
    }
}
