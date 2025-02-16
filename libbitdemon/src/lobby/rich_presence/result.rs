use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

pub struct RichPresenceInfoResult {
    pub is_online: bool,
    pub rich_presence_data: Vec<u8>,
}

impl BdSerialize for RichPresenceInfoResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_bool(self.is_online)?;
        writer.write_blob(self.rich_presence_data.as_ref())?;

        Ok(())
    }
}

impl From<Option<Vec<u8>>> for RichPresenceInfoResult {
    fn from(value: Option<Vec<u8>>) -> Self {
        if let Some(rich_presence_data) = value {
            RichPresenceInfoResult {
                is_online: true,
                rich_presence_data,
            }
        } else {
            RichPresenceInfoResult {
                is_online: false,
                rich_presence_data: Vec::new(),
            }
        }
    }
}
