use bitdemon::domain::title::Title;
use bitdemon::messaging::bd_serialization::BdSerialize;
use bitdemon::messaging::bd_writer::BdWriter;
use bitdemon::messaging::StreamMode;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;
use snafu::{ensure, Snafu};
use std::error::Error;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum BdAuthTicketType {
    UserToServiceTicket = 0x0,
    HostToServiceTicket = 0x1,
    UserToHostTicket = 0x2,
}

pub struct AuthTicket {
    pub ticket_type: BdAuthTicketType,
    pub title: Title,
    pub time_issued: u32,
    pub time_expires: u32,
    pub license_id: u64,
    pub user_id: u64,
    pub username: String,
    pub session_key: [u8; 24],
}

const MAGIC_NUMBER: u32 = 0xEFBDADDE;
const NAME_MAX_LEN: usize = 64;

#[derive(Debug, Snafu)]
#[snafu(display("Name too long when serializing auth ticket (len={name_len} max={NAME_MAX_LEN})"))]
struct UsernameTooLongError {
    name_len: usize,
}

impl BdSerialize for AuthTicket {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.set_type_checked(false);
        writer.set_mode(StreamMode::ByteMode);

        writer.write_u32(MAGIC_NUMBER)?;
        writer.write_u8(self.ticket_type.to_u8().unwrap())?;
        writer.write_u32(self.title.to_u32().unwrap())?;
        writer.write_u32(self.time_issued)?;
        writer.write_u32(self.time_expires)?;
        writer.write_u64(self.license_id)?;
        writer.write_u64(self.user_id)?;

        ensure!(
            self.username.len() <= NAME_MAX_LEN,
            UsernameTooLongSnafu {
                name_len: self.username.len()
            }
        );

        writer.write_bytes(self.username.as_ref())?;
        for _ in self.username.len()..64 {
            writer.write_bytes(&[0])?;
        }

        writer.write_bytes(self.session_key.as_ref())?;

        // Random hash stuff that is unused?
        writer.write_bytes(&[0, 0, 0, 0])?;
        Ok(())
    }
}
