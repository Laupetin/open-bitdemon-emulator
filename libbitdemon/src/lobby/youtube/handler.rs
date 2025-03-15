use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::youtube::result::YoutubeBoolResult;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::{info, warn};
use num_traits::FromPrimitive;
use std::error::Error;

pub struct YoutubeHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum YoutubeTaskId {
    // GetUploadStats
    StartAccountRegistration = 1,
    IsRegistered = 2,
    Unregister = 3,
    UploadVideo = 4,
    GetUserToken = 6,
}

impl LobbyHandler for YoutubeHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = YoutubeTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                .to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            YoutubeTaskId::StartAccountRegistration => {
                Self::start_account_registration(session, &mut message.reader)
            }
            YoutubeTaskId::IsRegistered => Self::is_registered(session, &mut message.reader),
            YoutubeTaskId::Unregister => Self::unregister(session, &mut message.reader),
            YoutubeTaskId::UploadVideo => Self::upload_video(session, &mut message.reader),
            YoutubeTaskId::GetUserToken => Self::get_user_token(session, &mut message.reader),
        }
    }
}

impl Default for YoutubeHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl YoutubeHandler {
    pub fn new() -> YoutubeHandler {
        YoutubeHandler {}
    }

    fn start_account_registration(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        info!("Trying to start account registration");

        TaskReply::with_only_error_code(
            BdErrorCode::YoutubeServiceError,
            YoutubeTaskId::StartAccountRegistration,
        )
        .to_response()
    }

    fn is_registered(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_results(
            YoutubeTaskId::IsRegistered,
            vec![Box::new(YoutubeBoolResult { value: false })],
        )
        .to_response()
    }

    fn unregister(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        info!("Trying to unregister");

        TaskReply::with_only_error_code(BdErrorCode::NoError, YoutubeTaskId::Unregister)
            .to_response()
    }

    fn upload_video(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let file_id = reader.read_u64()?;
        let is_private = reader.read_bool()?;
        let developer_tag_count = reader.read_u32()?;
        let mut developer_tags = Vec::with_capacity(developer_tag_count as usize);

        for _ in 0..developer_tag_count {
            developer_tags.push(reader.read_str()?);
        }

        info!("Trying to upload file {file_id} (private={is_private}; developerTags={developer_tags:?})");

        TaskReply::with_only_error_code(
            BdErrorCode::YoutubeServiceError,
            YoutubeTaskId::UploadVideo,
        )
        .to_response()
    }

    fn get_user_token(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        info!("Trying to get user token");

        TaskReply::with_only_error_code(
            BdErrorCode::YoutubeServiceError,
            YoutubeTaskId::GetUserToken,
        )
        .to_response()
    }
}
