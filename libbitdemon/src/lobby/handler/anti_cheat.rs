use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::BdErrorCode::NoError;
use crate::networking::bd_session::BdSession;
use log::{debug, warn};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use std::error::Error;

pub struct AntiCheatHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum AntiCheatTaskId {
    AnswerChallenges = 2,
    ReportConsoleId = 3, // Index is a guess
    ReportConsoleDetails = 4,
}

impl LobbyHandler for AntiCheatHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = AntiCheatTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!(
                "[Session {}] Client called unknown task {task_id_value}",
                session.id
            );
            return Ok(TaskReply::with_only_error_code(NoError, task_id_value).to_response()?);
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            AntiCheatTaskId::ReportConsoleDetails => {
                Self::report_console_details(session, &mut message.reader)
            }
            AntiCheatTaskId::AnswerChallenges | AntiCheatTaskId::ReportConsoleId => {
                warn!(
                    "[Session {}] Client called unimplemented task {task_id:?}",
                    session.id
                );
                Ok(TaskReply::with_only_error_code(NoError, task_id).to_response()?)
            }
        }
    }
}

impl AntiCheatHandler {
    pub fn new() -> AntiCheatHandler {
        AntiCheatHandler {}
    }

    fn report_console_details(
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let _blob1 = reader.read_blob()?; // Always blob with length 16 on PC with first 4 byte being 0x756B5B3
        let _uint1 = reader.read_u32()?; // Always 2 on PC
        let changelist = reader.read_u32()?; // Changelist of the game executable
        let _ulong1 = reader.read_u64()?; // Always 0 on PC
        let _ulong2 = reader.read_u64()?; // Always 0 on PC
        let _ulong3 = reader.read_u64()?; // Always 0 on PC
        let _blob2 = reader.read_blob()?; // Always nulled blob with length 6 on PC

        debug!(
            "[Session {}] Client reported console details changelist={changelist}",
            session.id
        );

        Ok(
            TaskReply::with_only_error_code(NoError, AntiCheatTaskId::ReportConsoleDetails)
                .to_response()?,
        )
    }
}
