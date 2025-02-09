use crate::networking::bd_session::BdSession;
use byteorder::{LittleEndian, WriteBytesExt};
use snafu::{ensure, Snafu};
use std::error::Error;
use std::io::Write;

pub struct BdResponse {
    should_encrypt: bool,
    data: Vec<u8>,
}

pub trait ResponseCreator {
    fn to_response(&self) -> Result<BdResponse, Box<dyn Error>>;
}

#[derive(Debug, Snafu)]
enum BdResponseError {
    #[snafu(display("Tried to send encrypted response but no session key is available"))]
    NoSessionKeyAvailableError,
}

impl BdResponse {
    pub fn unencrypted(data: Vec<u8>) -> Self {
        BdResponse {
            should_encrypt: false,
            data,
        }
    }
    pub fn encrypted_if_available(data: Vec<u8>) -> Self {
        BdResponse {
            should_encrypt: true,
            data,
        }
    }

    pub fn send(&self, session: &mut BdSession) -> Result<(), Box<dyn Error>> {
        if self.should_encrypt && session.session_key.is_some() {
            ensure!(session.session_key.is_some(), NoSessionKeyAvailableSnafu {});
            todo!();
        } else {
            // Written length minus length field itself
            let message_length = self.data.len() + 1;
            session.write_u32::<LittleEndian>(message_length as u32)?;
            session.write_u8(0u8)?; // Encrypted
            session.write(self.data.as_slice())?;
        }

        Ok(())
    }
}
