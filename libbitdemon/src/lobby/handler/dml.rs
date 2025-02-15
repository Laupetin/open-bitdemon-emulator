use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
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
            DmlTaskId::RecordIp => Self::record_ip(session, &mut message.reader),
            DmlTaskId::GetUserData => Self::get_user_data(session, &mut message.reader),
            DmlTaskId::GetUserHierarchicalData => {
                Self::get_user_hierarchical_data(session, &mut message.reader)
            }
        }
    }
}

impl DmlHandler {
    pub fn new() -> DmlHandler {
        DmlHandler {}
    }

    fn record_ip(
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let ip = reader.read_u32()?;
        info!("[Session {}] Recording IP: {ip}", session.id);

        Ok(
            TaskReply::with_only_error_code(BdErrorCode::NoError, DmlTaskId::RecordIp)
                .to_response()?,
        )
    }

    fn get_user_data(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let dml_info = Self::create_mock_dml_info();
        Ok(
            TaskReply::with_results(DmlTaskId::GetUserData, vec![Box::from(dml_info)])
                .to_response()?,
        )
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

        Ok(TaskReply::with_results(
            DmlTaskId::GetUserData,
            vec![Box::from(dml_hierarchical_info)],
        )
        .to_response()?)
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

struct DmlInfoResult {
    pub country_code: String,
    pub country: String,
    pub region: String,
    pub city: String,
    pub latitude: f32,
    pub longitude: f32,
}

struct DmlHierarchicalInfoResult {
    pub base: DmlInfoResult,
    pub tier0: u32,
    pub tier1: u32,
    pub tier2: u32,
    pub tier3: u32,
}

impl BdSerialize for DmlInfoResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_str(self.country_code.as_str())?;
        writer.write_str(self.country.as_str())?;
        writer.write_str(self.region.as_str())?;
        writer.write_str(self.city.as_str())?;
        writer.write_f32(self.latitude)?;
        writer.write_f32(self.longitude)?;

        Ok(())
    }
}

impl BdSerialize for DmlHierarchicalInfoResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        self.base.serialize(writer)?;
        writer.write_u32(self.tier0)?;
        writer.write_u32(self.tier1)?;
        writer.write_u32(self.tier2)?;
        writer.write_u32(self.tier3)?;

        Ok(())
    }
}
