use bitdemon::domain::result_slice::ResultSlice;
use bitdemon::lobby::content_streaming::{
    ContentStreamingServiceError, PublisherContentStreamingService, StreamInfo,
};
use bitdemon::networking::bd_session::BdSession;

pub struct DwPublisherContentStreamingService {}

impl PublisherContentStreamingService for DwPublisherContentStreamingService {
    fn get_publisher_stream_by_id(
        &self,
        session: &BdSession,
        file_id: u64,
    ) -> Result<StreamInfo, ContentStreamingServiceError> {
        todo!()
    }

    fn list_publisher_streams(
        &self,
        session: &BdSession,
        min_date_time: i64,
        category: u16,
        item_offset: usize,
        item_count: usize,
    ) -> Result<ResultSlice<StreamInfo>, ContentStreamingServiceError> {
        todo!()
    }

    fn filter_publisher_streams(
        &self,
        session: &BdSession,
        min_date_time: i64,
        category: u16,
        item_offset: usize,
        item_count: usize,
        filter: String,
    ) -> Result<ResultSlice<StreamInfo>, ContentStreamingServiceError> {
        todo!()
    }
}

impl DwPublisherContentStreamingService {
    pub fn new() -> DwPublisherContentStreamingService {
        DwPublisherContentStreamingService {}
    }
}
