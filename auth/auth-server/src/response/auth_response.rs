﻿use crate::auth_handler::AuthMessageType;
use bitdemon::messaging::bd_response::BdResponse;
use bitdemon::messaging::bd_writer::BdWriter;
use bitdemon::messaging::{BdErrorCode, StreamMode};
use num_traits::ToPrimitive;
use std::error::Error;

pub trait AuthResponse {
    fn message_type(&self) -> AuthMessageType;
    fn error_code(&self) -> BdErrorCode;
    fn write_auth_data(&self, _writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

impl dyn AuthResponse {
    pub fn response(&self) -> Result<BdResponse, Box<dyn Error>> {
        let mut buf = Vec::new();

        {
            let mut writer = BdWriter::new(&mut buf);
            writer.set_type_checked(false);
            writer.set_mode(StreamMode::BitMode);

            writer.write_u8(self.message_type().to_u8().unwrap())?;

            writer.set_type_checked(true);
            writer.write_type_checked_bit()?;

            writer.write_u32(self.error_code().to_u32().unwrap())?;

            self.write_auth_data(&mut writer)?;
        }

        Ok(BdResponse::new(buf))
    }
}
