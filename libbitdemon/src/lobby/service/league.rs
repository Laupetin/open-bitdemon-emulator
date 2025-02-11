use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::warn;
use num_traits::FromPrimitive;
use snafu::Snafu;
use std::error::Error;

pub struct LeagueHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum LeagueTaskId {
    // SetTeamIcon
    // GetTeamLeaguesAndSubdivisions
    // IncrementGamesPlayedCount
    GetTeamId = 1,
    GetTeamIDsForUser = 2,
    GetTeamSubdivisions = 3,
    SetTeamName = 4,

    // ? = 5
    GetTeamInfos = 6,
    GetTeamMemberInfos = 8,
    GetTeamSubdivisionInfos = 20,
    GetTeamSubdivisionHistory = 21,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum OrderType {
    OrderByTeamId = 0x0,
    OrderByRecentActivity = 0x1,
}

#[derive(Debug, Snafu)]
enum LeagueHandlerError {
    #[snafu(display("Value is not a valid order type (value={value})"))]
    InvalidOrderTypeError { value: u8 },
}

impl LobbyHandler for LeagueHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = LeagueTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!(
                "[Session {}] Client called unknown task {task_id_value}",
                session.id
            );
            return Ok(
                TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                    .to_response()?,
            );
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            LeagueTaskId::GetTeamId => Self::get_team_id(session, &mut message.reader),
            LeagueTaskId::GetTeamIDsForUser => {
                Self::get_team_ids_for_user(session, &mut message.reader)
            }
            LeagueTaskId::GetTeamSubdivisions => {
                Self::get_team_subdivisions(session, &mut message.reader)
            }
            LeagueTaskId::SetTeamName => Self::set_team_name(session, &mut message.reader),
            LeagueTaskId::GetTeamInfos => Self::get_team_infos(session, &mut message.reader),
            LeagueTaskId::GetTeamMemberInfos => {
                Self::get_team_member_infos(session, &mut message.reader)
            }
            LeagueTaskId::GetTeamSubdivisionInfos => {
                Self::get_team_subdivision_infos(session, &mut message.reader)
            }
            LeagueTaskId::GetTeamSubdivisionHistory => {
                Self::get_team_subdivision_history(session, &mut message.reader)
            }
        }
    }
}

impl LeagueHandler {
    pub fn new() -> LeagueHandler {
        LeagueHandler {}
    }

    fn get_team_id(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let _user_ids = reader.read_u64_array()?;

        // TODO: Do something useful

        Ok(
            TaskReply::with_only_error_code(BdErrorCode::NoError, LeagueTaskId::GetTeamId)
                .to_response()?,
        )
    }
    fn get_team_ids_for_user(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let _user_id = reader.read_u64()?;
        let order_type_value = reader.read_u8()?;
        let _order_type = OrderType::from_u8(order_type_value).ok_or_else(|| {
            InvalidOrderTypeSnafu {
                value: order_type_value,
            }
            .build()
        })?;
        let _offset = reader.read_u32()?;
        let _max_results = reader.read_u32()?;

        // TODO: Do something useful

        Ok(
            TaskReply::with_only_error_code(BdErrorCode::NoError, LeagueTaskId::GetTeamIDsForUser)
                .to_response()?,
        )
    }
    fn get_team_subdivisions(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let _team_id = reader.read_u64()?;
        let _league_ids = reader.read_u64_array()?;

        // TODO: Do something useful

        Ok(
            TaskReply::with_only_error_code(
                BdErrorCode::NoError,
                LeagueTaskId::GetTeamSubdivisions,
            )
            .to_response()?,
        )
    }
    fn set_team_name(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let _team_id = reader.read_u64()?;
        let _name = reader.read_str()?;

        // TODO: Do something useful

        Ok(
            TaskReply::with_only_error_code(BdErrorCode::NoError, LeagueTaskId::SetTeamName)
                .to_response()?,
        )
    }
    fn get_team_infos(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let _team_ids = reader.read_u64_array()?;

        // TODO: Do something useful

        Ok(
            TaskReply::with_only_error_code(BdErrorCode::NoError, LeagueTaskId::GetTeamInfos)
                .to_response()?,
        )
    }
    fn get_team_member_infos(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let _team_ids = reader.read_u64_array()?;

        // TODO: Do something useful

        Ok(
            TaskReply::with_only_error_code(BdErrorCode::NoError, LeagueTaskId::GetTeamMemberInfos)
                .to_response()?,
        )
    }
    fn get_team_subdivision_infos(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let _subdivision_ids = reader.read_u64_array()?;

        // TODO: Do something useful

        Ok(TaskReply::with_only_error_code(
            BdErrorCode::NoError,
            LeagueTaskId::GetTeamSubdivisionInfos,
        )
        .to_response()?)
    }
    fn get_team_subdivision_history(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let _team_id = reader.read_u64()?;
        let _league_id = reader.read_u64()?;
        let _season_ids = reader.read_u64_array()?;

        // TODO: Do something useful

        Ok(TaskReply::with_only_error_code(
            BdErrorCode::NoError,
            LeagueTaskId::GetTeamSubdivisionHistory,
        )
        .to_response()?)
    }
}
