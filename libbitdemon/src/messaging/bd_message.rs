use crate::crypto::{decrypt_buffer_in_place, encrypt_buffer_in_place, generate_iv_from_seed};
use crate::messaging::bd_reader::BdReader;
use crate::networking::bd_session::BdSession;
use snafu::{ensure, Snafu};
use std::error::Error;

pub struct BdMessage {
    pub reader: BdReader,
}

#[derive(Debug, Snafu)]
enum BdMessageError {
    #[snafu(display("Received encrypted message but no session key was set"))]
    NoSessionKeyError,
}

impl BdMessage {
    pub fn new(session: &BdSession, mut buf: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let encrypted = buf.get(0).unwrap();
        if *encrypted > 0 {
            ensure!(session.session_key.is_some(), NoSessionKeySnafu {});
            let seed = u32::from_le_bytes(buf[1..5].try_into().unwrap());

            let iv = generate_iv_from_seed(seed);
            let buf_len = buf.len();
            decrypt_buffer_in_place(
                &mut buf[5..buf_len],
                session.session_key.as_ref().unwrap(),
                &iv,
            )?;

            // TODO: Check hmac
            let _hmac = u32::from_le_bytes(buf[5..9].try_into().unwrap());

            Ok(BdMessage {
                reader: BdReader::new(Vec::from(&buf[9..buf.len()])),
            })
        } else {
            Ok(BdMessage {
                reader: BdReader::new(Vec::from(&buf[1..buf.len()])),
            })
        }
    }
}
