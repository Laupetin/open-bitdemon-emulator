use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

pub trait BdSerialize {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>>;
}

pub trait BdDeserialize {
    fn deserialize(reader: &mut BdReader) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}
