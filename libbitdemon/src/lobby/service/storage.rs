use crate::domain::result_slice::ResultSlice;
use crate::domain::title::Title;
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_serialization::{BdDeserialize, BdSerialize};
use crate::messaging::bd_writer::BdWriter;
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::warn;
use num_traits::FromPrimitive;
use snafu::Snafu;
use std::error::Error;
use std::sync::Arc;

/// Contains metadata describing a file that is stored by the backend.
#[derive(Clone)]
pub struct StorageFileInfo {
    /// The id of the file.
    /// Must be unique across all files the owner of the file owns.
    /// May or may not be unique across all users.
    /// May or may not be unique across all titles.
    id: u64,
    /// The name of the stored file.
    /// It may contain an extension or path separators.
    filename: String,
    /// The title the file was uploaded for.
    title: Title,
    /// The size of the file in bytes.
    file_size: i64,
    /// The seconds timestamp of when the file was initially uploaded or created.
    created: i64,
    /// The seconds timestamp of when the file was last modified.
    /// Must be greater or equal to the creation timestamp.
    modified: i64,
    /// The visibility level of the file.
    visibility: FileVisibility,
    /// The id of the user that owns the file.
    owner_id: u64,
}

/// Determines the visibility of a file
#[derive(PartialEq, Copy, Clone)]
pub enum FileVisibility {
    /// The file is visible for any logged-in user.
    VisiblePublic = 0,
    /// The file can only be seen by the user that owns it.
    VisiblePrivate = 1,
}

/// Errors that may occur when handling storage calls.
pub enum StorageServiceError {
    /// The authenticated user does not have permission to perform the requested operation.
    PermissionDeniedError,
    /// The name of the file is too long to process.
    FilenameTooLongError,
    /// The file is too long to process.
    StorageFileTooLargeError,
    /// The file does not exist.
    StorageFileNotFoundError,
}

pub type ThreadSafeStorageService = dyn StorageService + Sync + Send;

/// Implements domain logic for the storage handler.
pub trait StorageService {
    /// Retrieves the data of a file identified by an id.
    ///
    /// The owner is **NOT** necessarily the user that tries to retrieve the file.
    /// For the acting user reference the `session` parameter.
    /// The returned result contains details about the uploaded file.
    ///
    /// # Errors
    ///
    /// * [`PermissionDeniedError`][1]: The requested operation is not allowed for the current user.
    /// * [`StorageFileNotFoundError`][2]: The requested file could not be found.
    ///
    /// [1]: StorageServiceError::PermissionDeniedError
    /// [2]: StorageServiceError::StorageFileNotFoundError
    fn get_storage_file_data_by_id(
        &self,
        session: &BdSession,
        owner_id: u64,
        file_id: u64,
    ) -> Result<Vec<u8>, StorageServiceError>;

    /// Retrieves the data of a file identified by a filename.
    ///
    /// The owner is **NOT** necessarily the user that tries to retrieve the file.
    /// For the acting user reference the `session` parameter.
    /// The returned result contains details about the uploaded file.
    ///
    /// # Errors
    ///
    /// * [`PermissionDeniedError`][1]: The requested operation is not allowed for the current user.
    /// * [`StorageFileNotFoundError`][2]: The requested file could not be found.
    ///
    /// [1]: StorageServiceError::PermissionDeniedError
    /// [2]: StorageServiceError::StorageFileNotFoundError
    fn get_storage_file_data_by_name(
        &self,
        session: &BdSession,
        owner_id: u64,
        filename: String,
    ) -> Result<Vec<u8>, StorageServiceError>;

    /// Lists file details owned by a specified user.
    /// The result is returned as a [`ResultSlice`].
    ///
    /// The owner is **NOT** necessarily the user that tries to list the files.
    /// For the acting user reference the `session` parameter.
    /// The returned result contains details about the uploaded file.
    ///
    /// The `item_offset` parameter describes the amount of items to skip and **NOT** an index of a page.
    /// The amount of returned items should be equal or less than the value of the `item_count` parameter.
    ///
    /// The `min_date_time` parameter describes the lower bound of when the files need to be created on.
    /// Any files older than the specified timestamp should be excluded from the results.
    ///
    /// # Errors
    ///
    /// * [`PermissionDeniedError`][1]: The requested operation is not allowed for the current user.
    ///
    /// [1]: StorageServiceError::PermissionDeniedError
    fn list_storage_files(
        &self,
        session: &BdSession,
        owner_id: u64,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError>;

    /// Lists file details of files matching a specified filter owned by a specified user.
    /// The result is returned as a [`ResultSlice`].
    ///
    /// The owner is **NOT** necessarily the user that tries to list the files.
    /// For the acting user reference the `session` parameter.
    /// The returned result contains details about the uploaded file.
    ///
    /// The `item_offset` parameter describes the amount of items to skip and **NOT** an index of a page.
    /// The amount of returned items should be equal or less than the value of the `item_count` parameter.
    ///
    /// The `min_date_time` parameter describes the lower bound of when the files need to be created on.
    /// Any files older than the specified timestamp should be excluded from the results.
    ///
    /// The `filter` parameter specifies a string that the matches files must _start_ with.
    ///
    /// # Errors
    ///
    /// * [`PermissionDeniedError`][1]: The requested operation is not allowed for the current user.
    ///
    /// [1]: StorageServiceError::PermissionDeniedError
    fn filter_storage_files(
        &self,
        session: &BdSession,
        owner_id: u64,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
        filter: String,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError>;

    /// Processes and saves a file uploaded by a user.
    ///
    /// The owner is **NOT** necessarily the user that uploaded the file.
    /// For the acting user reference the `session` parameter.
    /// The returned result contains details about the uploaded file.
    ///
    /// # Errors
    ///
    /// * [`PermissionDeniedError`][1]: The requested operation is not allowed for the current user.
    /// * [`FilenameTooLongError`][2]: The name of the file is longer than allowed.
    /// * [`StorageFileTooLargeError`][3]: The size of the file is larger than allowed.
    ///
    /// [1]: StorageServiceError::PermissionDeniedError
    /// [2]: StorageServiceError::FilenameTooLongError
    /// [3]: StorageServiceError::StorageFileTooLargeError
    fn create_storage_file(
        &self,
        session: &BdSession,
        owner_id: u64,
        filename: String,
        visibility: FileVisibility,
        file_data: Vec<u8>,
    ) -> Result<StorageFileInfo, StorageServiceError>;

    /// Updates the data of a file that was previously created.
    ///
    /// The owner is **NOT** necessarily the user that tries to delete the file.
    /// For the acting user reference the `session` parameter.
    /// The returned result contains details about the uploaded file.
    ///
    /// # Errors
    ///
    /// * [`PermissionDeniedError`][1]: The requested operation is not allowed for the current user.
    /// * [`StorageFileNotFoundError`][2]: The requested file could not be found.
    /// * [`StorageFileTooLargeException`][3]: The requested file could not be found.
    ///
    /// [1]: StorageServiceError::PermissionDeniedError
    /// [2]: StorageServiceError::StorageFileNotFoundError
    /// [3]: StorageServiceError::StorageFileTooLargeException
    fn update_storage_file_data(
        &self,
        session: &BdSession,
        owner_id: u64,
        file_id: u64,
        file_data: Vec<u8>,
    ) -> Result<(), StorageServiceError>;

    /// Deletes a specified file.
    ///
    /// The owner is **NOT** necessarily the user that tries to delete the file.
    /// For the acting user reference the `session` parameter.
    /// The returned result contains details about the uploaded file.
    ///
    /// # Errors
    ///
    /// * [`PermissionDeniedError`][1]: The requested operation is not allowed for the current user.
    /// * [`StorageFileNotFoundError`][2]: The requested file could not be found.
    ///
    /// [1]: StorageServiceError::PermissionDeniedError
    /// [2]: StorageServiceError::StorageFileNotFoundError
    fn remove_storage_file(
        &self,
        session: &BdSession,
        owner_id: u64,
        filename: String,
    ) -> Result<(), StorageServiceError>;
}

pub type ThreadSafePublisherStorageService = dyn PublisherStorageService + Sync + Send;

pub trait PublisherStorageService {
    /// Gets the data of a specified publisher file.
    ///
    /// # Errors
    ///
    /// * [`PermissionDeniedError`][1]: The requested operation is not allowed for the current user.
    /// * [`StorageFileNotFoundError`][2]: The requested file could not be found.
    ///
    /// [1]: StorageServiceError::PermissionDeniedError
    /// [2]: StorageServiceError::StorageFileNotFoundError
    fn get_publisher_file_data(
        &self,
        session: &BdSession,
        filename: String,
    ) -> Result<Vec<u8>, StorageServiceError>;

    /// Lists details of the publisher files.
    /// The result is returned as a [`ResultSlice`].
    ///
    /// The `item_offset` parameter describes the amount of items to skip and **NOT** an index of a page.
    /// The amount of returned items should be equal or less than the value of the `item_count` parameter.
    ///
    /// The `min_date_time` parameter describes the lower bound of when the files need to be created on.
    /// Any files older than the specified timestamp should be excluded from the results.
    ///
    /// # Errors
    ///
    /// * [`PermissionDeniedError`][1]: The requested operation is not allowed for the current user.
    ///
    /// [1]: StorageServiceError::PermissionDeniedError
    fn list_publisher_files(
        &self,
        session: &BdSession,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError>;

    /// Lists details of the files of the publisher files.
    /// The result is returned as a [`ResultSlice`].
    ///
    /// The `item_offset` parameter describes the amount of items to skip and **NOT** an index of a page.
    /// The amount of returned items should be equal or less than the value of the `item_count` parameter.
    ///
    /// The `min_date_time` parameter describes the lower bound of when the files need to be created on.
    /// Any files older than the specified timestamp should be excluded from the results.
    ///
    /// The `filter` parameter specifies a string that the matches files must _start_ with.
    ///
    /// # Errors
    ///
    /// * [`PermissionDeniedError`][1]: The requested operation is not allowed for the current user.
    ///
    /// [1]: StorageServiceError::PermissionDeniedError
    fn filter_publisher_files(
        &self,
        session: &BdSession,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
        filter: String,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError>;
}

pub struct StorageHandler {
    storage_service: Arc<ThreadSafeStorageService>,
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

#[derive(Debug, Snafu)]
enum StorageHandlerError {
    #[snafu(display("Value is not a valid order type (value={value})"))]
    InvalidOrderTypeError { value: u8 },
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
            warn!(
                "[Session {}] Client called unknown task {task_id_value}",
                session.id
            );
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
                warn!(
                    "[Session {}] Client called unimplemented task {task_id:?}",
                    session.id
                );
                Ok(TaskReply::with_only_error_code(BdErrorCode::NoError, task_id).to_response()?)
            }
        }
    }
}

impl StorageHandler {
    pub fn new(
        storage_service: Arc<ThreadSafeStorageService>,
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
            .get_publisher_file_data(session, filename);

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

impl BdSerialize for StorageFileInfo {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u32(self.file_size as u32)?;
        writer.write_u64(self.id)?;
        writer.write_u32((self.created % (u32::MAX as i64)) as u32)?;
        writer.write_bool(self.visibility == FileVisibility::VisiblePrivate)?;
        writer.write_u64(self.owner_id)?;
        writer.write_str(self.filename.as_str())?;

        Ok(())
    }
}

struct FileDataResult {
    data: Vec<u8>,
}

impl BdDeserialize for FileDataResult {
    fn deserialize(reader: &mut BdReader) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let data = reader.read_blob()?;

        Ok(FileDataResult { data })
    }
}

impl BdSerialize for FileDataResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_blob(self.data.as_slice())
    }
}
