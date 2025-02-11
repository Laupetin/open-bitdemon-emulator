use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use crate::messaging::BdErrorCode::NoError;
use crate::networking::bd_session::BdSession;
use log::warn;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use std::error::Error;

pub struct TitleUtilitiesHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum TitleUtilitiesTaskId {
    // SendOwnedContent
    // GetMAC
    // GetUserIDs
    // SetEventLog
    VerifyString = 1,
    GetTitleStats = 2,
    RecordEvent = 3, // Deprecated for EventLog
    RecordIp = 4,
    RecordEventBin = 5, // Deprecated for EventLog
    GetServerTime = 6,
    AreUsersOnline = 7,
    GetUserNames = 9,
}

impl LobbyHandler for TitleUtilitiesHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = TitleUtilitiesTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!(
                "[Session {}] Client called unknown task {task_id_value}",
                session.id
            );
            return Ok(TaskReply::with_only_error_code(NoError, task_id_value).to_response()?);
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            TitleUtilitiesTaskId::GetServerTime => Self::get_server_time(),
            TitleUtilitiesTaskId::VerifyString
            | TitleUtilitiesTaskId::GetTitleStats
            | TitleUtilitiesTaskId::RecordEvent
            | TitleUtilitiesTaskId::RecordIp
            | TitleUtilitiesTaskId::RecordEventBin
            | TitleUtilitiesTaskId::AreUsersOnline
            | TitleUtilitiesTaskId::GetUserNames => {
                warn!(
                    "[Session {}] Client called unimplemented task {task_id:?}",
                    session.id
                );
                Ok(TaskReply::with_only_error_code(NoError, task_id).to_response()?)
            }
        }
    }
}

impl TitleUtilitiesHandler {
    pub fn new() -> TitleUtilitiesHandler {
        TitleUtilitiesHandler {}
    }

    fn get_server_time() -> Result<BdResponse, Box<dyn Error>> {
        let now = chrono::Utc::now().timestamp();
        let result = Box::from(TimestampResult {
            value: (now % (u32::MAX as i64)) as u32,
        });

        Ok(
            TaskReply::with_results(TitleUtilitiesTaskId::GetServerTime, vec![result])
                .to_response()?,
        )
    }
}

struct TimestampResult {
    value: u32,
}

impl BdSerialize for TimestampResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u32(self.value)
    }
}
