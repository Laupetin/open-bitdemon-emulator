use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::twitch::result::TwitchBoolResult;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::{info, warn};
use num_traits::FromPrimitive;
use std::error::Error;

pub struct TwitchHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum TwitchTaskId {
    LinkAccount = 1,
    UnlinkAccount = 2,
    IsLinked = 3,
    GetUserInfo = 4,
}

impl LobbyHandler for TwitchHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = TwitchTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                .to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            TwitchTaskId::LinkAccount => Self::link_account(session, &mut message.reader),
            TwitchTaskId::UnlinkAccount => Self::unlink_account(session, &mut message.reader),
            TwitchTaskId::IsLinked => Self::is_linked(session, &mut message.reader),
            TwitchTaskId::GetUserInfo => Self::get_user_info(session, &mut message.reader),
        }
    }
}

impl Default for TwitchHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TwitchHandler {
    pub fn new() -> TwitchHandler {
        TwitchHandler {}
    }

    fn link_account(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let token = reader.read_str()?;

        info!("Trying to link account token={token}");

        TaskReply::with_only_error_code(BdErrorCode::NoError, TwitchTaskId::LinkAccount)
            .to_response()
    }

    fn unlink_account(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        info!("Trying to unlink account");

        TaskReply::with_only_error_code(BdErrorCode::NoError, TwitchTaskId::LinkAccount)
            .to_response()
    }

    fn is_linked(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_results(
            TwitchTaskId::IsLinked,
            vec![Box::new(TwitchBoolResult { value: false })],
        )
        .to_response()
    }

    fn get_user_info(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        info!("Trying to get user token");

        TaskReply::with_only_error_code(BdErrorCode::ServiceNotAvailable, TwitchTaskId::GetUserInfo)
            .to_response()
    }
}
