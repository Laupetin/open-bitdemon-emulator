use crate::lobby::response::BdMessageType;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use crate::messaging::{BdErrorCode, StreamMode};
use num_traits::ToPrimitive;
use std::error::Error;

pub struct TaskReply {
    transaction_id: u64,
    error_code: BdErrorCode,
    operation_id: u8,
    results: Vec<Box<dyn BdSerialize>>,
    total_num_results: Option<u32>,
}

impl TaskReply {
    pub fn with_only_error_code(error_code: BdErrorCode) -> TaskReply {
        TaskReply {
            transaction_id: 0u64,
            error_code,
            operation_id: 0u8,
            results: Vec::new(),
            total_num_results: None,
        }
    }
}

impl ResponseCreator for TaskReply {
    fn to_response(&self) -> Result<BdResponse, Box<dyn Error>> {
        let mut data = Vec::new();

        {
            let mut writer = BdWriter::new(&mut data);
            writer.set_type_checked(false);
            writer.set_mode(StreamMode::ByteMode);

            writer.write_u8(BdMessageType::LobbyServiceTaskReply.to_u8().unwrap())?;

            writer.set_type_checked(true);

            writer.write_u64(self.transaction_id)?;
            writer.write_u32(self.error_code.to_u32().unwrap())?;
            writer.write_u8(self.operation_id)?;

            // numResults
            writer.write_u32(self.results.len() as u32)?;

            // totalNumResults
            writer.write_u32(
                self.total_num_results
                    .unwrap_or_else(|| self.results.len() as u32),
            )?;

            for result in &self.results {
                result.serialize(&mut writer)?;
            }
        }

        Ok(BdResponse::encrypted_if_available(data))
    }
}
