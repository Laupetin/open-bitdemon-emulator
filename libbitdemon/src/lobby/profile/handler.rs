use crate::lobby::profile::{ProfileServiceError, ThreadSafeProfileService};
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

pub struct ProfileHandler {
    pub profile_service: Arc<ThreadSafeProfileService>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum ProfileTaskId {
    GetPublicInfos = 1,
    GetPrivateInfo = 2,
    SetPublicInfo = 3,
    SetPrivateInfo = 4,
    DeleteProfile = 5,
}

impl LobbyHandler for ProfileHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = ProfileTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                .to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            ProfileTaskId::GetPublicInfos => self.get_public_infos(session, &mut message.reader),
            ProfileTaskId::GetPrivateInfo => self.get_private_infos(session, &mut message.reader),
            ProfileTaskId::SetPublicInfo => self.set_public_info(session, &mut message.reader),
            ProfileTaskId::SetPrivateInfo => self.set_private_info(session, &mut message.reader),
            ProfileTaskId::DeleteProfile => self.delete_profile(session, &mut message.reader),
        }
    }
}

impl ProfileHandler {
    pub fn new(profile_service: Arc<ThreadSafeProfileService>) -> ProfileHandler {
        ProfileHandler { profile_service }
    }

    fn get_public_infos(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let mut user_ids = Vec::new();

        while reader.next_is_u64()? {
            user_ids.push(reader.read_u64()?);
        }

        let result = self.profile_service.get_public_profiles(session, user_ids);

        match result {
            Ok(profile_infos) => Ok(TaskReply::with_results(
                ProfileTaskId::GetPublicInfos,
                profile_infos
                    .into_iter()
                    .map(|profile_info| Box::from(profile_info) as Box<dyn BdSerialize>)
                    .collect(),
            )
            .to_response()?),
            Err(code) => Self::handle_profile_error(code, ProfileTaskId::GetPublicInfos)?,
        }
    }

    fn get_private_infos(
        &self,
        session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let result = self.profile_service.get_private_profile(session);

        match result {
            Ok(profile_info) => Ok(TaskReply::with_results(
                ProfileTaskId::GetPublicInfos,
                vec![Box::from(profile_info)],
            )
            .to_response()?),
            Err(code) => Self::handle_profile_error(code, ProfileTaskId::GetPublicInfos)?,
        }
    }

    fn set_public_info(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let mut profile_data = vec![0; reader.remaining_bytes()?];
        reader.read_bytes(profile_data.as_mut_slice())?;

        let result = self
            .profile_service
            .set_public_profile(session, profile_data);

        match result {
            Ok(_) => Ok(TaskReply::with_only_error_code(
                BdErrorCode::NoError,
                ProfileTaskId::SetPublicInfo,
            )
            .to_response()?),
            Err(code) => Self::handle_profile_error(code, ProfileTaskId::SetPublicInfo)?,
        }
    }

    fn set_private_info(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let mut profile_data = vec![0; reader.remaining_bytes()?];
        reader.read_bytes(profile_data.as_mut_slice())?;

        let result = self
            .profile_service
            .set_private_profile(session, profile_data);

        match result {
            Ok(_) => Ok(TaskReply::with_only_error_code(
                BdErrorCode::NoError,
                ProfileTaskId::SetPrivateInfo,
            )
            .to_response()?),
            Err(code) => Self::handle_profile_error(code, ProfileTaskId::SetPrivateInfo)?,
        }
    }

    fn delete_profile(
        &self,
        session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let result = self.profile_service.delete_profile(session);

        match result {
            Ok(_) => Ok(TaskReply::with_only_error_code(
                BdErrorCode::NoError,
                ProfileTaskId::DeleteProfile,
            )
            .to_response()?),
            Err(code) => Self::handle_profile_error(code, ProfileTaskId::DeleteProfile)?,
        }
    }

    fn handle_profile_error(
        code: ProfileServiceError,
        task_id: ProfileTaskId,
    ) -> Result<Result<BdResponse, Box<dyn Error>>, Box<dyn Error>> {
        Ok(Ok(TaskReply::with_only_error_code(
            match code {
                ProfileServiceError::PermissionDenied => BdErrorCode::PermissionDenied,
                ProfileServiceError::NoProfileInfoFound => BdErrorCode::NoProfileInfoExists,
            },
            task_id,
        )
        .to_response()?))
    }
}
