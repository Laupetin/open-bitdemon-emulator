use crate::response::BdMessageType::LsgServiceConnectionId;
use bitdemon::messaging::bd_response::{BdResponse, ResponseCreator};
use bitdemon::messaging::bd_writer::BdWriter;
use bitdemon::messaging::StreamMode::ByteMode;
use num_traits::ToPrimitive;
use std::error::Error;

pub struct LsgServiceTaskReply {}

pub struct LsgErrorResponse {}

pub struct ConnectionIdResponse {
    connection_id: u64,
}

impl ConnectionIdResponse {
    pub fn new(connection_id: u64) -> ConnectionIdResponse {
        ConnectionIdResponse { connection_id }
    }
}

impl ResponseCreator for ConnectionIdResponse {
    fn to_response(&self) -> Result<BdResponse, Box<dyn Error>> {
        let mut data = Vec::new();
        {
            let mut writer = BdWriter::new(&mut data);
            writer.set_type_checked(false);
            writer.set_mode(ByteMode);

            writer.write_u8(LsgServiceConnectionId.to_u8().unwrap())?;

            writer.set_type_checked(true);

            writer.write_u64(self.connection_id)?;
        }

        Ok(BdResponse::encrypted_if_available(data))
    }
}
