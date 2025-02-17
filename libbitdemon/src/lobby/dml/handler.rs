use crate::lobby::dml::result::{DmlHierarchicalInfoResult, DmlInfoResult};
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::{info, warn};
use num_traits::FromPrimitive;
use std::error::Error;

pub struct DmlHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum DmlTaskId {
    RecordIp = 1,
    GetUserData = 2,
    GetUserHierarchicalData = 3,
}

impl LobbyHandler for DmlHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = DmlTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                .to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            DmlTaskId::RecordIp => Self::record_ip(session, &mut message.reader),
            DmlTaskId::GetUserData => Self::get_user_data(session, &mut message.reader),
            DmlTaskId::GetUserHierarchicalData => {
                Self::get_user_hierarchical_data(session, &mut message.reader)
            }
        }
    }
}

impl Default for DmlHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DmlHandler {
    pub fn new() -> DmlHandler {
        DmlHandler {}
    }

    fn record_ip(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let ip = reader.read_u32()?;
        info!("Recording IP: {ip}");

        TaskReply::with_only_error_code(BdErrorCode::NoError, DmlTaskId::RecordIp).to_response()
    }

    fn get_user_data(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let dml_info = Self::create_mock_dml_info();

        TaskReply::with_results(DmlTaskId::GetUserData, vec![Box::from(dml_info)]).to_response()
    }

    fn get_user_hierarchical_data(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let dml_hierarchical_info = DmlHierarchicalInfoResult {
            base: Self::create_mock_dml_info(),
            tier0: 0,
            tier1: 0,
            tier2: 0,
            tier3: 0,
        };

        TaskReply::with_results(
            DmlTaskId::GetUserData,
            vec![Box::from(dml_hierarchical_info)],
        )
        .to_response()
    }
}

impl DmlHandler {
    fn create_mock_dml_info() -> DmlInfoResult {
        DmlInfoResult {
            country_code: String::from("US"),
            country: String::from("United States"),
            region: String::from("California"),
            city: String::from("Los Angeles"),
            latitude: 34.0453f32,
            longitude: -118.2413f32,
        }
    }
}
