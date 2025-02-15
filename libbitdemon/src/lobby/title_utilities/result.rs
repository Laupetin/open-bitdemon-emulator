use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

pub struct TimestampResult {
    pub value: u32,
}

impl BdSerialize for TimestampResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u32(self.value)
    }
}
