use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

pub struct TwitchBoolResult {
    pub value: bool,
}

impl BdSerialize for TwitchBoolResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_bool(self.value)
    }
}
