use crate::lobby::profile::ProfileInfo;
use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

impl BdSerialize for ProfileInfo {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u64(self.user_id)?;
        writer.write_bytes(self.data.as_ref())
    }
}
