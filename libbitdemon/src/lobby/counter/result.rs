use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_serialization::{BdDeserialize, BdSerialize};
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

#[derive(Debug)]
pub struct CounterValueResult {
    pub counter_id: u32,
    pub counter_value: i64,
}

impl BdSerialize for CounterValueResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u32(self.counter_id)?;
        writer.write_i64(self.counter_value)?;

        Ok(())
    }
}

impl BdDeserialize for CounterValueResult {
    fn deserialize(reader: &mut BdReader) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let counter_id = reader.read_u32()?;
        let counter_value = reader.read_i64()?;

        Ok(CounterValueResult {
            counter_id,
            counter_value,
        })
    }
}
