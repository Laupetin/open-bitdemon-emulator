use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::rich_presence::result::RichPresenceInfoResult;
use crate::lobby::rich_presence::{RichPresenceServiceError, ThreadSafeRichPresenceService};
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

pub struct RichPresenceHandler {
    pub rich_presence_service: Arc<ThreadSafeRichPresenceService>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum RichPresenceTaskId {
    SetInfo = 1,
    GetInfo = 2,
}

impl LobbyHandler for RichPresenceHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = RichPresenceTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return Ok(
                TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                    .to_response()?,
            );
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            RichPresenceTaskId::SetInfo => self.set_info(session, &mut message.reader),
            RichPresenceTaskId::GetInfo => self.get_info(session, &mut message.reader),
        }
    }
}

impl RichPresenceHandler {
    pub fn new(rich_presence_service: Arc<ThreadSafeRichPresenceService>) -> RichPresenceHandler {
        RichPresenceHandler {
            rich_presence_service,
        }
    }

    fn set_info(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let mut user_id = reader.read_u64()?;
        if user_id == 0 {
            user_id = session.authentication().unwrap().user_id;
        }

        let data = reader.read_blob()?;

        let result = self.rich_presence_service.set_info(session, user_id, data);

        match result {
            Ok(_) => Ok(TaskReply::with_only_error_code(
                BdErrorCode::NoError,
                RichPresenceTaskId::SetInfo,
            )
            .to_response()?),
            Err(code) => Self::handle_rich_presence_error(code, RichPresenceTaskId::SetInfo)?,
        }
    }

    fn get_info(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let mut user_ids = Vec::new();
        while reader.next_is_u64().unwrap_or(false) {
            user_ids.push(reader.read_u64()?);
        }

        let result = self
            .rich_presence_service
            .get_info(session, user_ids.as_ref())
            .map(|user_presence_list| {
                user_presence_list
                    .into_iter()
                    .map(|user_presence| {
                        Box::from(RichPresenceInfoResult::from(user_presence))
                            as Box<dyn BdSerialize>
                    })
                    .collect::<Vec<Box<dyn BdSerialize>>>()
            });

        match result {
            Ok(_) => Ok(TaskReply::with_only_error_code(
                BdErrorCode::NoError,
                RichPresenceTaskId::GetInfo,
            )
            .to_response()?),
            Err(code) => Self::handle_rich_presence_error(code, RichPresenceTaskId::GetInfo)?,
        }
    }

    fn handle_rich_presence_error(
        code: RichPresenceServiceError,
        task_id: RichPresenceTaskId,
    ) -> Result<Result<BdResponse, Box<dyn Error>>, Box<dyn Error>> {
        Ok(Ok(TaskReply::with_only_error_code(
            match code {
                RichPresenceServiceError::PermissionDeniedError => BdErrorCode::PermissionDenied,
                RichPresenceServiceError::RichPresenceDataTooLargeError => {
                    BdErrorCode::RichPresenceDataTooLarge
                }
                RichPresenceServiceError::TooManyUsersError => {
                    BdErrorCode::RichPresenceTooManyUsers
                }
            },
            task_id,
        )
        .to_response()?))
    }
}
