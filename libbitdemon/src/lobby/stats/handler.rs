use crate::lobby::LobbyHandler;
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::stats::result::StatsInfoResultOut;
use crate::messaging::BdErrorCode;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_serialization::BdSerialize;
use crate::networking::bd_session::BdSession;
use log::warn;
use num_traits::FromPrimitive;
use std::error::Error;

pub struct StatsHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum StatsTaskId {
    WriteStats = 1,
    ReadStatsByEntityId = 3,
    ReadStatsByRank = 4,
    ReadStatsByPivot = 5,
    ReadStatsByLeaderboardIdsAndEntityIds = 11,
}

impl LobbyHandler for StatsHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = StatsTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                .to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            StatsTaskId::WriteStats => self.write_stats(session, &mut message.reader),
            StatsTaskId::ReadStatsByEntityId => {
                self.read_stats_by_entity_id(session, &mut message.reader)
            }
            StatsTaskId::ReadStatsByRank => self.read_stats_by_rank(session, &mut message.reader),
            StatsTaskId::ReadStatsByPivot => self.read_stats_by_pivot(session, &mut message.reader),
            StatsTaskId::ReadStatsByLeaderboardIdsAndEntityIds => {
                self.read_stats_by_leaderboard_ids_and_entity_ids(session, &mut message.reader)
            }
        }
    }
}

impl StatsHandler {
    pub fn new() -> StatsHandler {
        StatsHandler {}
    }

    fn write_stats(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, StatsTaskId::WriteStats).to_response()
    }

    fn read_stats_by_entity_id(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let _leaderboard_id = reader.read_u32()?;

        let authentication = session
            .authentication()
            .expect("Expect username to be there");

        let mut results: Vec<Box<dyn BdSerialize>> = vec![];
        while let Ok(entity_id) = reader.read_u64() {
            if entity_id == authentication.user_id {
                results.push(Box::new(StatsInfoResultOut {
                    entity_id,
                    rating: 2,
                    rank: 1,
                    entity_name: authentication.username.clone(),
                    seconds_since_update: 1337,
                }));
            }
        }

        if results.is_empty() {
            TaskReply::with_only_error_code(
                BdErrorCode::NoStatsForUser,
                StatsTaskId::ReadStatsByEntityId,
            )
            .to_response()
        } else {
            TaskReply::with_results(StatsTaskId::ReadStatsByEntityId, results).to_response()
        }
    }

    fn read_stats_by_rank(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, StatsTaskId::ReadStatsByRank)
            .to_response()
    }

    fn read_stats_by_pivot(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(BdErrorCode::NoError, StatsTaskId::ReadStatsByPivot)
            .to_response()
    }

    fn read_stats_by_leaderboard_ids_and_entity_ids(
        &self,
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        TaskReply::with_only_error_code(
            BdErrorCode::NoError,
            StatsTaskId::ReadStatsByLeaderboardIdsAndEntityIds,
        )
        .to_response()
    }
}
