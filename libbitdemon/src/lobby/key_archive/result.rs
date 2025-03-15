use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_serialization::BdDeserialize;
use num_traits::FromPrimitive;
use snafu::Snafu;
use std::error::Error;

#[derive(Debug, Snafu)]
enum KeyArchiveResultError {
    #[snafu(display("Value is not a valid update type (value={value})"))]
    InvalidUpdateType { value: u8 },
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum KeyArchiveUpdateType {
    Replace = 0,
    Add = 1,
    Max = 2,
    Min = 3,
    And = 4,
    Or = 5,
    Xor = 6,
    SubSafe = 7,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct KeyValuePairWriteResult {
    pub index: u16,
    pub value: i64,
    pub update_type: KeyArchiveUpdateType,
}

impl BdDeserialize for KeyValuePairWriteResult {
    fn deserialize(reader: &mut BdReader) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let index = reader.read_u16()?;
        let value = reader.read_i64()?;
        let update_type_value = reader.read_u8()?;

        let update_type = KeyArchiveUpdateType::from_u8(update_type_value).ok_or_else(|| {
            InvalidUpdateTypeSnafu {
                value: update_type_value,
            }
            .build()
        })?;

        Ok(KeyValuePairWriteResult {
            index,
            value,
            update_type,
        })
    }
}
