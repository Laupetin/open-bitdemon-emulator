use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

pub struct GroupCountResult {
    pub group_id: u32,
    pub group_count: u32,
}

impl BdSerialize for GroupCountResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u32(self.group_id)?;
        writer.write_u32(self.group_count)?;

        Ok(())
    }
}
