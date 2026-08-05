use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

#[derive(Debug)]
pub struct BdSessionIdResult {
    pub session_id: u64,
}

impl BdSerialize for BdSessionIdResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u64(self.session_id)?;

        Ok(())
    }
}
