use crate::auth_handler::AuthMessageType;
use bitdemon::messaging::bd_response::{BdResponse, ResponseCreator};
use bitdemon::messaging::bd_writer::BdWriter;
use bitdemon::messaging::{BdErrorCode, StreamMode};
use num_traits::ToPrimitive;
use std::error::Error;

pub trait AuthResponse {
    fn message_type(&self) -> AuthMessageType;
    fn error_code(&self) -> BdErrorCode;
    fn write_auth_data(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>>;
}

impl ResponseCreator for dyn AuthResponse {
    fn to_response(&self) -> Result<BdResponse, Box<dyn Error>> {
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

        Ok(BdResponse::unencrypted(buf))
    }
}

pub struct AuthResponseWithOnlyCode {
    message_type: AuthMessageType,
    error_code: BdErrorCode,
}

impl AuthResponseWithOnlyCode {
    pub fn new(message_type: AuthMessageType, error_code: BdErrorCode) -> AuthResponseWithOnlyCode {
        AuthResponseWithOnlyCode {
            message_type,
            error_code,
        }
    }
}

impl AuthResponse for AuthResponseWithOnlyCode {
    fn message_type(&self) -> AuthMessageType {
        self.message_type
    }

    fn error_code(&self) -> BdErrorCode {
        self.error_code
    }

    fn write_auth_data(&self, _writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
