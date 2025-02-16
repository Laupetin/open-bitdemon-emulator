use crate::lobby::group::result::GroupCountResult;
use crate::lobby::group::ThreadSafeGroupService;
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::warn;
use num_traits::FromPrimitive;
use std::error::Error;
use std::sync::Arc;

pub struct GroupHandler {
    pub group_service: Arc<ThreadSafeGroupService>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum GroupTaskId {
    SetGroups = 1,
    SetGroupsForEntity = 2,
    GetEntityGroups = 3,
    GetGroupCounts = 4,
}

impl LobbyHandler for GroupHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = GroupTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return Ok(
                TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                    .to_response()?,
            );
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            GroupTaskId::SetGroups => self.set_groups(session, &mut message.reader),
            GroupTaskId::GetGroupCounts => self.get_group_counts(session, &mut message.reader),
            GroupTaskId::GetEntityGroups | GroupTaskId::SetGroupsForEntity => {
                warn!("Client called unimplemented task {task_id:?}");
                Ok(TaskReply::with_only_error_code(BdErrorCode::NoError, task_id).to_response()?)
            }
        }
    }
}

impl GroupHandler {
    pub fn new(group_service: Arc<ThreadSafeGroupService>) -> GroupHandler {
        GroupHandler { group_service }
    }

    fn set_groups(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let groups = reader.read_u32_array()?;

        self.group_service.set_groups(session, &groups)?;

        Ok(
            TaskReply::with_only_error_code(BdErrorCode::NoError, GroupTaskId::SetGroups)
                .to_response()?,
        )
    }

    fn get_group_counts(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let groups = reader.read_u32_array()?;
        let group_count = groups.len();

        let counts = self.group_service.get_group_counts(session, &groups)?;
        debug_assert_eq!(group_count, counts.len());

        let results: Vec<Box<dyn BdSerialize>> = (0..group_count)
            .map(|i| {
                Box::from(GroupCountResult {
                    group_id: groups[i],
                    group_count: counts[i].max(u32::MAX as u64) as u32,
                }) as Box<dyn BdSerialize>
            })
            .collect();

        Ok(TaskReply::with_results(GroupTaskId::GetGroupCounts, results).to_response()?)
    }
}
