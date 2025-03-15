use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_serialization::BdDeserialize;
use std::error::Error;

pub struct EventInfo {
    pub category_id: u32,
    pub binary_data: Option<Vec<u8>>,
    pub string_data: Option<String>,
}

impl BdDeserialize for EventInfo {
    fn deserialize(reader: &mut BdReader) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let category_id = reader.read_u32()?;
        let is_binary = reader.read_bool()?;

        if is_binary {
            let binary_data = reader.read_blob()?;

            Ok(EventInfo {
                category_id,
                binary_data: Some(binary_data),
                string_data: None,
            })
        } else {
            let string_data = reader.read_str()?;

            Ok(EventInfo {
                category_id,
                binary_data: None,
                string_data: Some(string_data),
            })
        }
    }
}
