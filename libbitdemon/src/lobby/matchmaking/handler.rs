use crate::lobby::matchmaking::ThreadSafeMatchmakingService;
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::warn;
use num_traits::FromPrimitive;
use std::error::Error;
use std::sync::Arc;

pub struct MatchmakingHandler {
    pub matchmaking_service: Arc<ThreadSafeMatchmakingService>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum MatchmakingTaskId {
    // FindSessionsFromIds
    CreateSession = 1,
    UpdateSession = 2,
    DeleteSession = 3,
    FindSessionFromId = 4,
    FindSessions = 5,
    NotifyJoin = 6,
    NotifyLeave = 7,
    InviteToSession = 8,
    SubmitPerformance = 9,
    GetPerformanceValues = 10,
    GetSessionInvites = 11,
    UpdateSessionPlayers = 12, // x2 <-> x1 notifySessionInviteRecv
    FindSessionsPaged = 13,
    FindSessionsByEntityIds = 14,
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
            MatchmakingTaskId::FindSessionFromId => {
                self.find_session_from_id(session, &mut message.reader)
            }
            MatchmakingTaskId::FindSessions => self.find_sessions(session, &mut message.reader),
            MatchmakingTaskId::NotifyJoin => self.notify_join(session, &mut message.reader),
            MatchmakingTaskId::NotifyLeave => self.notify_leave(session, &mut message.reader),
            MatchmakingTaskId::InviteToSession => {
                self.invite_to_session(session, &mut message.reader)
            }
            MatchmakingTaskId::SubmitPerformance => {
                self.submit_performance(session, &mut message.reader)
            }
            MatchmakingTaskId::GetPerformanceValues => {
                self.get_performance_values(session, &mut message.reader)
            }
            MatchmakingTaskId::GetSessionInvites => {
                self.get_session_invites(session, &mut message.reader)
            }
            MatchmakingTaskId::UpdateSessionPlayers => {
                self.update_session_players(session, &mut message.reader)
            }
            MatchmakingTaskId::FindSessionsPaged => {
                self.find_sessions_paged(session, &mut message.reader)
            }
            MatchmakingTaskId::FindSessionsByEntityIds => {
                self.find_sessions_by_entity_ids(session, &mut message.reader)
            }
        }
    }
}

impl MatchmakingHandler {
    pub fn new(matchmaking_service: Arc<ThreadSafeMatchmakingService>) -> MatchmakingHandler {
        MatchmakingHandler {
            matchmaking_service,
        }
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
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::UpdateSession)
            .to_response()
    }

    fn delete_session(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::DeleteSession)
            .to_response()
    }

    fn find_session_from_id(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::FindSessionFromId)
            .to_response()
    }

    fn find_sessions(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::FindSessions)
            .to_response()
    }

    fn notify_join(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::NotifyJoin)
            .to_response()
    }

    fn notify_leave(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::NotifyLeave)
            .to_response()
    }

    fn invite_to_session(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::InviteToSession)
            .to_response()
    }

    fn submit_performance(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::SubmitPerformance)
            .to_response()
    }

    fn get_performance_values(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(
            BdErrorCode::NoError,
            MatchmakingTaskId::GetPerformanceValues,
        )
        .to_response()
    }

    fn get_session_invites(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::GetSessionInvites)
            .to_response()
    }

    fn update_session_players(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(
            BdErrorCode::NoError,
            MatchmakingTaskId::UpdateSessionPlayers,
        )
        .to_response()
    }

    fn find_sessions_paged(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, MatchmakingTaskId::FindSessionsPaged)
            .to_response()
    }

    fn find_sessions_by_entity_ids(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(
            BdErrorCode::NoError,
            MatchmakingTaskId::FindSessionsByEntityIds,
        )
        .to_response()
    }
}
