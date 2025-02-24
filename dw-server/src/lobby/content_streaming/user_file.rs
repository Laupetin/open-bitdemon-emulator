use bitdemon::domain::result_slice::ResultSlice;
use bitdemon::lobby::content_streaming::{
    ContentStreamingServiceError, StreamCreationRequest, StreamInfo, StreamSlot, StreamUrl,
    UploadedStream, UserContentStreamingService,
};
use bitdemon::networking::bd_session::BdSession;

pub struct DwUserContentStreamingService {}

impl UserContentStreamingService for DwUserContentStreamingService {
    fn get_user_streams_by_id(
        &self,
        session: &BdSession,
        file_id: &[u64],
    ) -> Result<Vec<StreamInfo>, ContentStreamingServiceError> {
        todo!()
    }

    fn list_streams_of_users(
        &self,
        session: &BdSession,
        owner_ids: &[u64],
        min_date_time: i64,
        category: u16,
        item_offset: usize,
        item_count: usize,
    ) -> Result<ResultSlice<StreamInfo>, ContentStreamingServiceError> {
        todo!()
    }

    fn request_stream_upload(
        &self,
        session: &BdSession,
        request_data: StreamCreationRequest,
    ) -> Result<StreamUrl, ContentStreamingServiceError> {
        todo!()
    }

    fn finish_stream_upload(
        &self,
        session: &BdSession,
        uploaded_file: UploadedStream,
    ) -> Result<u64, ContentStreamingServiceError> {
        todo!()
    }

    fn request_stream_deletion(
        &self,
        session: &BdSession,
        slot_id: StreamSlot,
    ) -> Result<StreamUrl, ContentStreamingServiceError> {
        todo!()
    }
}

impl DwUserContentStreamingService {
    pub fn new() -> DwUserContentStreamingService {
        DwUserContentStreamingService {}
    }
}
