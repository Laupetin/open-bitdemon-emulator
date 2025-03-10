use crate::domain::result_slice::ResultSlice;
use crate::domain::title::Title;
use crate::networking::bd_session::BdSession;

/// The ID of a category that a stream is assigned to.
pub type CategoryId = u16;
/// The slot that is stream is in.
pub type StreamSlot = u16;

/// Contains metadata describing a file that can be streamed from the backend.
#[derive(Clone)]
pub struct StreamInfo {
    /// The id of the stream.
    /// Must be unique across all files of a title.
    pub id: u64,
    /// The name of the file.
    /// It may contain an extension or path separators.
    pub filename: String,
    /// The title the stream was uploaded for.
    pub title: Title,
    /// The size of the streamed file in bytes.
    pub stream_size: u64,
    /// The size of the summary in bytes. Details about the summary are unknown.
    pub summary_file_size: u64,
    /// The seconds timestamp of when the stream was initially uploaded or created.
    pub created: i64,
    /// The seconds timestamp of when the stream was last modified.
    /// Must be greater or equal to the creation timestamp.
    pub modified: i64,
    /// The id of the user that owns the stream.
    pub owner_id: u64,
    /// The last known name of the user that owns the stream.
    pub owner_name: String,
    /// The url under which the user can stream the file.
    pub url: String,
    /// Metadata that was set for the stream.
    pub metadata: Vec<u8>,
    /// The category that was set for the stream.
    pub category: CategoryId,
    /// The slot that the stream is in.
    pub slot: StreamSlot,
    /// The tags that were set for the stream.
    pub tags: Vec<StreamTag>,
    /// The amount of streams that were created by copying this stream.
    pub num_copies_made: u32,
    /// The id of the user that the stream was originally created from.
    pub origin_id: u64,
}

/// Describes a tag that can be set on a stream.
#[derive(Clone, Debug)]
pub struct StreamTag {
    pub primary: u64,
    pub secondary: u64,
}

/// The request of a user to create a stream.
#[derive(Clone, Debug)]
pub struct StreamCreationRequest {
    /// The filename of the stream that the user wants to create.
    pub filename: String,
    /// The slot that the stream should be assigned to.
    /// If the slot of the user already has a stream, it is replaced.
    pub slot: StreamSlot,
    /// The size of the stream.
    pub file_size: u64,
    /// The category of the stream.
    pub category: CategoryId,
    /// The checksum of the stream.
    pub checksum: Vec<u8>,
    /// The locale for the stream.
    pub client_locale: String,
}

/// Contains the url that the requested user operation can be performed at.
/// The request method depends on the operation that was requested.
#[derive(Clone)]
pub struct StreamUrl {
    /// The ID of the stream that the URL is for.
    pub stream_id: u64,
    /// The url that should be called to perform the requested operation.
    pub url: String,
    /// Unknown.
    pub server_type: u16,
    /// Unknown.
    pub server_index: String,
}

/// Contains data to finish the creation of a stream.
#[derive(Clone, Debug)]
pub struct UploadedStream {
    /// The filename of the stream that the user uploaded.
    pub filename: String,
    /// The slot that the stream should be assigned to.
    /// If the slot of the user already has a stream, it is replaced.
    pub slot: StreamSlot,
    /// Unknown.
    pub server_type: u16,
    /// Unknown.
    pub server_index: String,
    /// The size of the stream.
    pub file_size: u64,
    /// The category of the stream.
    pub category: CategoryId,
    /// Metadata that is attached to the stream.
    pub metadata: Vec<u8>,
    /// Tags that are attached to the stream.
    pub tags: Vec<StreamTag>,
    /// The locale for the stream.
    pub client_locale: String,
}

/// Errors that may occur when handling content streaming calls.
#[derive(Debug)]
pub enum ContentStreamingServiceError {
    /// The authenticated user does not have permission to perform the requested operation.
    PermissionDenied,
    /// The user has exceeded the storage space assigned to him.
    StorageSpaceExceeded,
    /// The user has uploaded too many streams.
    StreamCountExceeded,
    /// The name of the stream is too long to process.
    FilenameTooLong,
    /// The uploaded metadata is larger than allowed.
    MetaDataTooLarge,
    /// None of the requested streams could be found.
    NoStreamFound,
}

pub type ThreadSafeUserContentStreamingService = dyn UserContentStreamingService + Sync + Send;

/// Implements domain logic concerning user files of the storage service.
///
/// User files are files created by users of the service and are bound to the title they are created for.
/// If a file is private it can only be accessed by the user itself.
/// In case it is public, it can also be accessed by other users.
/// Users can create, read and delete files bound to their own user id.
pub trait UserContentStreamingService {
    /// Retrieves info for streams with specified IDs.
    /// A list of all found stream infos should be found unless no stream could be found.
    /// In that case, a [NoStreamFound](ContentStreamingServiceError::NoStreamFound) error should be returned.
    ///
    /// The specified url in the info will be called using a http `GET` request in case the user decides to stream the data.
    fn get_user_streams_by_id(
        &self,
        session: &BdSession,
        file_ids: &[u64],
    ) -> Result<Vec<StreamInfo>, ContentStreamingServiceError>;

    /// Retrieves info for streams of a specified user group.
    /// The returned streams must have a modification date that is newer or equal than `min_date_time`.
    /// They must be categorized with the specified category.
    /// The returned result slice should have the specified offset and count.
    ///
    /// The specified url in the info will be called using a http `GET` request in case the user decides to stream the data.
    fn list_streams_of_users(
        &self,
        session: &BdSession,
        owner_ids: &[u64],
        min_date_time: i64,
        category: u16,
        item_offset: usize,
        item_count: usize,
    ) -> Result<ResultSlice<StreamInfo>, ContentStreamingServiceError>;

    /// A user requested to upload a new stream.
    /// The data that the user specified for the upload is specified in `request_data`.
    /// The service is expected to return an url to which the user can send the stream data.
    ///
    /// The data will be sent to the specified http endpoint using the `PUT` request method.
    fn request_stream_upload(
        &self,
        session: &BdSession,
        request_data: StreamCreationRequest,
    ) -> Result<StreamUrl, ContentStreamingServiceError>;

    /// A user has successfully uploaded a new stream to a previously requested stream upload.
    /// The user sends complementary data to finish the stream creation process.
    /// The service is expected to return the ID of the newly created file.
    fn finish_stream_upload(
        &self,
        session: &BdSession,
        uploaded_file: UploadedStream,
    ) -> Result<u64, ContentStreamingServiceError>;

    /// A user requested to delete an existing stream that he previously uploaded.
    /// The service is expected to return an url the user can call to delete the stream.
    ///
    /// A request to the specified http endpoint will be made using the `DELETE` request method.
    fn request_stream_deletion(
        &self,
        session: &BdSession,
        slot_id: StreamSlot,
    ) -> Result<StreamUrl, ContentStreamingServiceError>;
}

pub type ThreadSafePublisherContentStreamingService =
    dyn PublisherContentStreamingService + Sync + Send;

/// Implements domain logic concerning publisher files.
///
/// Publisher files are files offered by the backend service provider for a certain title.
/// They can be read by any user that is authenticated for this title.
/// Users cannot create or overwrite publisher files.
pub trait PublisherContentStreamingService {
    /// Retrieves info for a publisher stream with specified ID.
    /// If the stream could not be found, a [NoStreamFound](ContentStreamingServiceError::NoStreamFound) error should be returned.
    ///
    /// The specified url in the info will be called using a http `GET` request in case the user decides to stream the data.
    fn get_publisher_stream_by_id(
        &self,
        session: &BdSession,
        file_id: u64,
    ) -> Result<StreamInfo, ContentStreamingServiceError>;

    /// Retrieves info for publisher streams.
    /// The returned streams must have a modification date that is newer or equal than `min_date_time`.
    /// They must be categorized with the specified category.
    /// The returned result slice should have the specified offset and count.
    ///
    /// The specified url in the info will be called using a http `GET` request in case the user decides to stream the data.
    fn list_publisher_streams(
        &self,
        session: &BdSession,
        min_date_time: i64,
        category: u16,
        item_offset: usize,
        item_count: usize,
    ) -> Result<ResultSlice<StreamInfo>, ContentStreamingServiceError>;

    /// Retrieves info for publisher streams using a filter value that any filename of a stream must begin with.
    /// The returned streams must have a modification date that is newer or equal than `min_date_time`.
    /// They must be categorized with the specified category.
    /// The returned result slice should have the specified offset and count.
    ///
    /// The specified url in the info will be called using a http `GET` request in case the user decides to stream the data.
    fn filter_publisher_streams(
        &self,
        session: &BdSession,
        min_date_time: i64,
        category: u16,
        item_offset: usize,
        item_count: usize,
        filter: String,
    ) -> Result<ResultSlice<StreamInfo>, ContentStreamingServiceError>;
}
