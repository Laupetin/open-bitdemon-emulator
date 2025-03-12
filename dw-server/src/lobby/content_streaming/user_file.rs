use crate::config::DwServerConfig;
use crate::lobby::content_streaming::db::{
    create_empty_stream, delete_db_stream, get_slot_count_for_upload, get_stream_data,
    get_stream_id_for_slot, get_streams_by_ids, get_streams_by_owners, record_user_name,
    set_stream_data, set_stream_metadata, PersistedStreamInfo,
};
use bitdemon::domain::result_slice::ResultSlice;
use bitdemon::domain::title::Title;
use bitdemon::lobby::content_streaming::{
    ContentStreamingServiceError, StreamCreationRequest, StreamInfo, StreamSlot, StreamUrl,
    UploadedStream, UserContentStreamingService,
};
use bitdemon::networking::bd_session::BdSession;
use chrono::Utc;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header};
use log::info;
use num_traits::ToPrimitive;
use rand::prelude::StdRng;
use rand::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum UserFileClaimOperation {
    Stream,
    Create,
    Delete,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserFileClaims {
    /// Expiration time (as UTC timestamp)
    pub exp: i64,
    /// Issued at (as UTC timestamp)
    pub iat: i64,
    /// Subject (whom token refers to)
    pub sub: String,
    /// ID of the title the operation is for
    pub stream_title: u32,
    /// ID of the file the operation is for
    pub stream_id: u64,
    /// Operation that is granted for the file
    pub stream_operation: UserFileClaimOperation,
}

pub struct DwUserContentStreamingService {
    content_server_hostname: String,
    content_server_port: u16,
    encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

const CLAIM_LIFETIME_IN_SECONDS: i64 = 5 * 60; // 5min
const MAX_FILENAME_LENGTH: usize = 260;
const MAX_USER_FILE_SIZE: usize = 50_000; // 50KB
const MAX_METADATA_SIZE: usize = 50_000; // 50KB
const MAX_SLOT_COUNT: usize = 128;

impl UserContentStreamingService for DwUserContentStreamingService {
    fn get_user_streams_by_id(
        &self,
        session: &BdSession,
        file_ids: &[u64],
    ) -> Result<Vec<StreamInfo>, ContentStreamingServiceError> {
        info!("Requesting stream file_ids={file_ids:?}");

        let authentication = session
            .authentication()
            .expect("session to be authentication checked");

        let res: Vec<StreamInfo> = get_streams_by_ids(authentication.title, file_ids)
            .into_iter()
            .map(|persisted_stream| self.build_get_url(authentication.user_id, persisted_stream))
            .collect();

        if !res.is_empty() {
            Ok(res)
        } else {
            Err(ContentStreamingServiceError::NoStreamFound)
        }
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
        info!("Listing streams of users={owner_ids:?}");

        let authentication = session
            .authentication()
            .expect("session to be authentication checked");

        let (res, total): (Vec<PersistedStreamInfo>, usize) = get_streams_by_owners(
            authentication.title,
            owner_ids,
            min_date_time,
            category,
            item_offset,
            item_count,
        );

        let res: Vec<StreamInfo> = res
            .into_iter()
            .map(|persisted_stream| self.build_get_url(authentication.user_id, persisted_stream))
            .collect();

        Ok(ResultSlice::with_total_count(res, item_offset, total))
    }

    fn request_stream_upload(
        &self,
        session: &BdSession,
        request_data: StreamCreationRequest,
    ) -> Result<StreamUrl, ContentStreamingServiceError> {
        info!("Requesting stream upload request={request_data:?}");

        if request_data.file_size as usize > MAX_USER_FILE_SIZE {
            return Err(ContentStreamingServiceError::StorageSpaceExceeded);
        }

        if request_data.filename.len() > MAX_FILENAME_LENGTH {
            return Err(ContentStreamingServiceError::StorageSpaceExceeded);
        }

        let authentication = session
            .authentication()
            .expect("session to be authentication checked");

        let slot_count_for_upload = get_slot_count_for_upload(
            authentication.title,
            authentication.user_id,
            request_data.slot,
        );

        if !slot_count_for_upload.given_slot_is_taken
            && slot_count_for_upload.used_slots >= MAX_SLOT_COUNT
        {
            return Err(ContentStreamingServiceError::StreamCountExceeded);
        }

        let stream_id = create_empty_stream(
            authentication.title,
            authentication.user_id,
            request_data.filename.as_str(),
            request_data.slot,
            request_data.category,
        );

        record_user_name(authentication.user_id, authentication.username.as_str());

        Ok(self.build_stream_url(
            authentication.user_id,
            authentication.title,
            stream_id,
            UserFileClaimOperation::Create,
        ))
    }

    fn finish_stream_upload(
        &self,
        session: &BdSession,
        uploaded_file: UploadedStream,
    ) -> Result<u64, ContentStreamingServiceError> {
        info!("Finishing stream upload={uploaded_file:?}");

        let authentication = session
            .authentication()
            .expect("session to be authentication checked");

        if uploaded_file.metadata.len() > MAX_METADATA_SIZE {
            return Err(ContentStreamingServiceError::MetaDataTooLarge);
        }

        set_stream_metadata(
            authentication.title,
            authentication.user_id,
            uploaded_file.slot,
            uploaded_file.metadata,
            uploaded_file.tags,
        )
        .map_err(|_| ContentStreamingServiceError::NoStreamFound)
    }

    fn request_stream_deletion(
        &self,
        session: &BdSession,
        slot_id: StreamSlot,
    ) -> Result<StreamUrl, ContentStreamingServiceError> {
        info!("Deleting stream slot={slot_id:?}");

        let authentication = session
            .authentication()
            .expect("session to be authentication checked");

        get_stream_id_for_slot(authentication.title, authentication.user_id, slot_id)
            .map(|stream_id| {
                self.build_stream_url(
                    authentication.user_id,
                    authentication.title,
                    stream_id,
                    UserFileClaimOperation::Delete,
                )
            })
            .map_err(|_| ContentStreamingServiceError::NoStreamFound)
    }
}

impl DwUserContentStreamingService {
    pub fn new(config: &DwServerConfig) -> DwUserContentStreamingService {
        let mut random = [0u8; 128];
        let mut rng = StdRng::from_os_rng();
        rng.fill_bytes(&mut random);

        let encoding_key = EncodingKey::from_secret(&random);
        let decoding_key = DecodingKey::from_secret(&random);

        DwUserContentStreamingService {
            content_server_hostname: config.hostname().to_string(),
            content_server_port: config.content_port(),
            encoding_key,
            decoding_key,
        }
    }

    pub fn stream_by_id(&self, title: Title, stream_id: u64) -> Option<Vec<u8>> {
        get_stream_data(title, stream_id)
    }

    pub fn set_stream_data(&self, title: Title, stream_id: u64, data: Vec<u8>) -> bool {
        set_stream_data(title, stream_id, data)
    }

    pub fn delete_stream(&self, title: Title, stream_id: u64) -> bool {
        delete_db_stream(title, stream_id).is_ok()
    }

    fn build_get_url(&self, user_id: u64, persisted_stream: PersistedStreamInfo) -> StreamInfo {
        let id = persisted_stream.id;
        let title_num = persisted_stream.title.to_u32().unwrap();

        let jwt = self.create_jwt(
            user_id,
            persisted_stream.title,
            persisted_stream.id,
            UserFileClaimOperation::Stream,
        );

        StreamInfo {
            id: persisted_stream.id,
            filename: persisted_stream.filename,
            title: persisted_stream.title,
            stream_size: persisted_stream.stream_size,
            summary_file_size: 0,
            created: persisted_stream.created,
            modified: persisted_stream.modified,
            owner_id: persisted_stream.owner_id,
            owner_name: persisted_stream.owner_name,
            url: format!(
                "http://{}:{}/content/user/{title_num}/{id}?authorization={jwt}",
                self.content_server_hostname, self.content_server_port
            ),
            metadata: persisted_stream.metadata,
            category: persisted_stream.category,
            slot: persisted_stream.slot,
            tags: persisted_stream.tags,
            num_copies_made: 0,
            origin_id: 0,
        }
    }

    fn build_stream_url(
        &self,
        user_id: u64,
        title: Title,
        stream_id: u64,
        operation: UserFileClaimOperation,
    ) -> StreamUrl {
        let title_num = title.to_u32().unwrap();
        let jwt = self.create_jwt(user_id, title, stream_id, operation);
        StreamUrl {
            stream_id,
            url: format!(
                "http://{}:{}/content/user/{title_num}/{stream_id}?authorization={jwt}",
                self.content_server_hostname, self.content_server_port
            ),
            server_type: 1,
            server_index: "".to_string(),
        }
    }

    fn create_jwt(
        &self,
        user_id: u64,
        title: Title,
        stream_id: u64,
        stream_operation: UserFileClaimOperation,
    ) -> String {
        let now = Utc::now().timestamp();
        let claims = UserFileClaims {
            exp: now + CLAIM_LIFETIME_IN_SECONDS,
            iat: now,
            sub: format!("{user_id}"),
            stream_title: title.to_u32().unwrap(),
            stream_id,
            stream_operation,
        };

        encode(&Header::default(), &claims, &self.encoding_key).expect("Jwt creation to work")
    }
}
