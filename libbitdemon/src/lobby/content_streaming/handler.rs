use crate::domain::result_slice::ResultSlice;
use crate::lobby::content_streaming::result::FileIdResult;
use crate::lobby::content_streaming::service::{
    ContentStreamingServiceError, ThreadSafePublisherContentStreamingService,
    ThreadSafeUserContentStreamingService,
};
use crate::lobby::content_streaming::{
    StreamCreationRequest, StreamInfo, StreamTag, StreamUrl, UploadedStream,
};
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::warn;
use num_traits::FromPrimitive;
use std::error::Error;
use std::sync::Arc;

pub struct ContentStreamingHandler {
    content_streaming_service: Arc<ThreadSafeUserContentStreamingService>,
    publisher_content_streaming_service: Arc<ThreadSafePublisherContentStreamingService>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum ContentStreamingTaskId {
    // GetQuotaUsage
    // ReportContent
    // RemoveFile
    // UploadUserSummaryMetaData
    // DownloadUserSummary
    // PreDownloadITunesPurchasedFile
    GetFileMetadataById = 1,
    ListFilesByOwner = 2,
    ListAllPublisherFiles = 3,
    PreUploadFile = 5,
    PostUploadFile = 6,
    PreDownloadFileBySlot = 7,
    PreDeleteFile = 8,
    PreDownloadByFileId = 9,
    PreDownloadPublisherFile = 10,

    // 11 = ?
    // 12 = ?
    ListFilesByOwners = 14,
    PreCopyFromPooledStorage = 15,
    PostCopy = 16,
    PreUploadSummary = 17,
    PostUploadSummary = 18,
    PreDownloadSummary = 19,
    PreCopyFromUserStorage = 20,
}

impl LobbyHandler for ContentStreamingHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = ContentStreamingTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                .to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            ContentStreamingTaskId::GetFileMetadataById => {
                self.get_file_metadata_by_id(session, &mut message.reader)
            }
            ContentStreamingTaskId::ListFilesByOwner => {
                self.list_files_by_owner(session, &mut message.reader)
            }
            ContentStreamingTaskId::ListAllPublisherFiles => {
                self.list_all_publisher_files(session, &mut message.reader)
            }
            ContentStreamingTaskId::PreUploadFile => {
                self.pre_upload_file(session, &mut message.reader)
            }
            ContentStreamingTaskId::PostUploadFile => {
                self.post_upload_file(session, &mut message.reader)
            }
            ContentStreamingTaskId::PreDeleteFile => {
                self.pre_delete_file(session, &mut message.reader)
            }
            ContentStreamingTaskId::PreDownloadByFileId => {
                self.pre_download_by_file_id(session, &mut message.reader)
            }
            ContentStreamingTaskId::PreDownloadPublisherFile => {
                self.pre_download_publisher_file(session, &mut message.reader)
            }
            ContentStreamingTaskId::ListFilesByOwners => {
                self.list_files_by_owners(session, &mut message.reader)
            }
            ContentStreamingTaskId::PreDownloadFileBySlot
            | ContentStreamingTaskId::PreCopyFromUserStorage
            | ContentStreamingTaskId::PreCopyFromPooledStorage
            | ContentStreamingTaskId::PostCopy
            | ContentStreamingTaskId::PreUploadSummary
            | ContentStreamingTaskId::PostUploadSummary
            | ContentStreamingTaskId::PreDownloadSummary => {
                warn!("Client called unimplemented task {task_id:?}");
                Ok(TaskReply::with_only_error_code(BdErrorCode::NoError, task_id).to_response()?)
            }
        }
    }
}

impl ContentStreamingHandler {
    pub fn new(
        content_streaming_service: Arc<ThreadSafeUserContentStreamingService>,
        publisher_content_streaming_service: Arc<ThreadSafePublisherContentStreamingService>,
    ) -> ContentStreamingHandler {
        ContentStreamingHandler {
            content_streaming_service,
            publisher_content_streaming_service,
        }
    }

    fn get_file_metadata_by_id(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let num_ids = reader.read_u32()?;

        let mut file_ids = Vec::with_capacity(num_ids as usize);
        for _ in 0..num_ids {
            file_ids.push(reader.read_u64()?);
        }

        let result = self
            .content_streaming_service
            .get_user_streams_by_id(session, file_ids.as_slice());

        match result {
            Ok(streams) => Ok(TaskReply::with_results(
                ContentStreamingTaskId::GetFileMetadataById,
                streams
                    .into_iter()
                    .map(|stream| Box::from(stream) as Box<dyn BdSerialize>)
                    .collect(),
            )
            .to_response()?),
            Err(error) => Ok(TaskReply::with_only_error_code(
                error.into(),
                ContentStreamingTaskId::GetFileMetadataById,
            )
            .to_response()?),
        }
    }

    fn list_files_by_owner(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let owner_id = reader.read_u64()?;
        let min_date_time = reader.read_u32()?;
        let item_count = reader.read_u16()?;
        let item_offset = reader.read_u16()?;
        let category_id = reader.read_u16()?;

        let result = self.content_streaming_service.list_streams_of_users(
            session,
            &[owner_id],
            min_date_time as i64,
            category_id,
            item_offset as usize,
            item_count as usize,
        );

        self.answer_for_stream_info_slice(ContentStreamingTaskId::ListFilesByOwner, result)
    }

    fn list_all_publisher_files(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let min_date_time = reader.read_u32()?;
        let item_count = reader.read_u16()?;
        let item_offset = reader.read_u16()?;
        let category_id = reader.read_u16()?;

        let result = if reader.next_is_str().unwrap_or(false) {
            let filter = reader.read_str()?;
            self.publisher_content_streaming_service
                .filter_publisher_streams(
                    session,
                    min_date_time as i64,
                    category_id,
                    item_offset as usize,
                    item_count as usize,
                    filter,
                )
        } else {
            self.publisher_content_streaming_service
                .list_publisher_streams(
                    session,
                    min_date_time as i64,
                    category_id,
                    item_offset as usize,
                    item_count as usize,
                )
        };

        self.answer_for_stream_info_slice(ContentStreamingTaskId::ListAllPublisherFiles, result)
    }

    fn pre_upload_file(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let filename = reader.read_str()?;
        let slot = reader.read_u16()?;
        let file_size = reader.read_u32()?;
        let category = reader.read_u16()?;
        let checksum = reader.read_blob()?;
        let client_locale = reader.read_str()?;

        let request_data = StreamCreationRequest {
            filename,
            slot,
            file_size: file_size as u64,
            category,
            checksum,
            client_locale,
        };

        let result = self
            .content_streaming_service
            .request_stream_upload(session, request_data);

        self.answer_for_stream_url(ContentStreamingTaskId::PreUploadFile, result)
    }

    fn post_upload_file(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let filename = reader.read_str()?;
        let slot = reader.read_u16()?;
        let server_type = reader.read_u16()?;
        let server_index = reader.read_str()?;
        let file_size = reader.read_u32()?;
        let category = reader.read_u16()?;
        let metadata = reader.read_blob()?;
        let tags_data = reader.read_u64_array()?;
        let client_locale = reader.read_str()?;

        let tag_count = tags_data.len() / 2;
        let mut tags = Vec::with_capacity(tag_count);
        for i in 0..tag_count {
            tags.push(StreamTag {
                primary: tags_data[i * 2],
                secondary: tags_data[i * 2 + 1],
            })
        }

        let uploaded_stream = UploadedStream {
            filename,
            slot,
            server_type,
            server_index,
            file_size: file_size as u64,
            category,
            metadata,
            tags,
            client_locale,
        };

        let result = self
            .content_streaming_service
            .finish_stream_upload(session, uploaded_stream);

        match result {
            Ok(file_id) => Ok(TaskReply::with_results(
                ContentStreamingTaskId::PostUploadFile,
                vec![Box::from(FileIdResult { id: file_id })],
            )
            .to_response()?),
            Err(error) => Ok(TaskReply::with_only_error_code(
                error.into(),
                ContentStreamingTaskId::PostUploadFile,
            )
            .to_response()?),
        }
    }

    fn pre_delete_file(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let slot = reader.read_u16()?;

        let result = self
            .content_streaming_service
            .request_stream_deletion(session, slot);

        self.answer_for_stream_url(ContentStreamingTaskId::PreDeleteFile, result)
    }

    fn pre_download_by_file_id(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let file_id = reader.read_u64()?;
        let _file_size = reader.read_u32()?;

        let result = self
            .content_streaming_service
            .get_user_streams_by_id(session, &[file_id]);

        match result {
            Ok(mut info) => {
                if let Some(first_stream) = info.pop() {
                    Ok(TaskReply::with_results(
                        ContentStreamingTaskId::PreDownloadByFileId,
                        vec![Box::from(first_stream)],
                    )
                    .to_response()?)
                } else {
                    Ok(TaskReply::with_only_error_code(
                        BdErrorCode::ContentStreamingFileNotAvailable,
                        ContentStreamingTaskId::PreDownloadByFileId,
                    )
                    .to_response()?)
                }
            }
            Err(error) => Ok(TaskReply::with_only_error_code(
                error.into(),
                ContentStreamingTaskId::PreDownloadByFileId,
            )
            .to_response()?),
        }
    }

    fn pre_download_publisher_file(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let file_id = reader.read_u64()?;
        let _file_size = reader.read_u32()?;

        let result = self
            .publisher_content_streaming_service
            .get_publisher_stream_by_id(session, file_id);

        match result {
            Ok(stream) => Ok(TaskReply::with_results(
                ContentStreamingTaskId::PreDownloadPublisherFile,
                vec![Box::from(stream)],
            )
            .to_response()?),
            Err(error) => Ok(TaskReply::with_only_error_code(
                error.into(),
                ContentStreamingTaskId::PreDownloadPublisherFile,
            )
            .to_response()?),
        }
    }

    fn list_files_by_owners(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let owner_ids = reader.read_u64_array()?;
        let min_date_time = reader.read_u32()?;
        let item_count = reader.read_u16()?;
        let item_offset = reader.read_u16()?;
        let category_id = reader.read_u16()?;

        let result = self.content_streaming_service.list_streams_of_users(
            session,
            owner_ids.as_slice(),
            min_date_time as i64,
            category_id,
            item_offset as usize,
            item_count as usize,
        );

        self.answer_for_stream_info_slice(ContentStreamingTaskId::ListFilesByOwners, result)
    }

    fn answer_for_stream_info_slice(
        &self,
        task_id: ContentStreamingTaskId,
        result: Result<ResultSlice<StreamInfo>, ContentStreamingServiceError>,
    ) -> Result<BdResponse, Box<dyn Error>> {
        match result {
            Ok(info) => {
                Ok(TaskReply::with_result_slice(task_id, info.serializable()).to_response()?)
            }
            Err(error) => Ok(TaskReply::with_only_error_code(error.into(), task_id).to_response()?),
        }
    }

    fn answer_for_stream_url(
        &self,
        task_id: ContentStreamingTaskId,
        result: Result<StreamUrl, ContentStreamingServiceError>,
    ) -> Result<BdResponse, Box<dyn Error>> {
        match result {
            Ok(url) => Ok(TaskReply::with_results(task_id, vec![Box::from(url)]).to_response()?),
            Err(error) => Ok(TaskReply::with_only_error_code(error.into(), task_id).to_response()?),
        }
    }
}

impl From<ContentStreamingServiceError> for BdErrorCode {
    fn from(value: ContentStreamingServiceError) -> Self {
        match value {
            ContentStreamingServiceError::PermissionDenied => BdErrorCode::PermissionDenied,
            ContentStreamingServiceError::FilenameTooLong => {
                BdErrorCode::ContentStreamingFilenameMaxLengthExceeded
            }
            ContentStreamingServiceError::StorageSpaceExceeded => {
                BdErrorCode::ContentStreamingStorageSpaceExceeded
            }
            ContentStreamingServiceError::StreamCountExceeded => {
                BdErrorCode::ContentStreamingNumFilesExceeded
            }
            ContentStreamingServiceError::MetaDataTooLarge => {
                BdErrorCode::ContentStreamingMaxThumbDataSizeExceeded
            }
            ContentStreamingServiceError::NoStreamFound => {
                BdErrorCode::ContentStreamingFileNotAvailable
            }
        }
    }
}
