use crate::lobby::LobbyHandler;
use crate::lobby::response::task_reply::TaskReply;
use crate::messaging::BdErrorCode;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::networking::bd_session::BdSession;
use log::warn;
use num_traits::FromPrimitive;
use std::error::Error;

pub struct MatchmakingHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum MatchmakingTaskId {
    CreateSession = 1,
    UpdateSession = 2,
    DeleteSession = 3,
    FindSessions = 5,
    SubmitPerformance = 9,
    GetPerformanceValues = 10,
    UpdateSessionPlayers = 12,
    FindSessionsPaged = 13,
}

impl LobbyHandler for MatchmakingHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = MatchmakingTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                .to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            MatchmakingTaskId::CreateSession => self.create_session(session, &mut message.reader),
            MatchmakingTaskId::UpdateSession => self.update_session(session, &mut message.reader),
            MatchmakingTaskId::DeleteSession => self.delete_session(session, &mut message.reader),
            MatchmakingTaskId::FindSessions => self.find_sessions(session, &mut message.reader),
            MatchmakingTaskId::SubmitPerformance => {
                self.submit_performance(session, &mut message.reader)
            }
            MatchmakingTaskId::GetPerformanceValues => {
                self.get_performance_values(session, &mut message.reader)
            }
            MatchmakingTaskId::UpdateSessionPlayers => {
                self.update_session_players(session, &mut message.reader)
            }
            MatchmakingTaskId::FindSessionsPaged => {
                self.find_sessions_paged(session, &mut message.reader)
            }
        }
    }
}

impl MatchmakingHandler {
    pub fn new() -> MatchmakingHandler {
        MatchmakingHandler {}
    }

    fn create_session(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::CreateSession)
            .to_response()
    }

    fn update_session(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::CreateSession)
            .to_response()
    }

    fn delete_session(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::CreateSession)
            .to_response()
    }

    fn find_sessions(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::CreateSession)
            .to_response()
    }

    fn submit_performance(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::CreateSession)
            .to_response()
    }

    fn get_performance_values(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::CreateSession)
            .to_response()
    }

    fn update_session_players(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::CreateSession)
            .to_response()
    }

    fn find_sessions_paged(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::CreateSession)
            .to_response()
    }
}
