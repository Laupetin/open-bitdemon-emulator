use crate::domain::result_slice::ResultSlice;
use crate::domain::title::Title;
use crate::networking::bd_session::BdSession;

/// Contains metadata describing a file that is stored by the backend.
#[derive(Clone)]
pub struct StorageFileInfo {
    /// The id of the file.
    /// Must be unique across all files the owner of the file owns.
    /// May or may not be unique across all users.
    /// May or may not be unique across all titles.
    pub id: u64,
    /// The name of the stored file.
    /// It may contain an extension or path separators.
    pub filename: String,
    /// The title the file was uploaded for.
    pub title: Title,
    /// The size of the file in bytes.
    pub file_size: u64,
    /// The seconds timestamp of when the file was initially uploaded or created.
    pub created: i64,
    /// The seconds timestamp of when the file was last modified.
    /// Must be greater or equal to the creation timestamp.
    pub modified: i64,
    /// The visibility level of the file.
    pub visibility: FileVisibility,
    /// The id of the user that owns the file.
    pub owner_id: u64,
}

/// Determines the visibility of a file
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FileVisibility {
    /// The file can only be seen by the user that owns it.
    VisiblePrivate,
    /// The file is visible for any logged-in user.
    VisiblePublic,
}

/// Errors that may occur when handling storage calls.
#[derive(Debug)]
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

pub type ThreadSafeUserStorageService = dyn UserStorageService + Sync + Send;

/// Implements domain logic concerning user files of the storage service.
///
/// User files are files created by users of the service and are bound to the title they are created for.
/// If a file is private it can only be accessed by the user itself.
/// In case it is public, it can also be accessed by other users.
/// Users can create, read and delete files bound to their own user id.
pub trait UserStorageService {
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

/// Implements domain logic concerning publisher files.
///
/// Publisher files are files offered by the backend service provider for a certain title.
/// They can be read by any user that is authenticated for this title.
/// Users cannot create or overwrite publisher files.
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
