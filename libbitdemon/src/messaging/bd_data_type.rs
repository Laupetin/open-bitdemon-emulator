use num_traits::{FromPrimitive, ToPrimitive};
use snafu::{OptionExt, Snafu};
use std::error::Error;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum BdDataType {
    NoType = 0x0,
    BoolType = 0x1,
    SignedChar8Type = 0x2,
    UnsignedChar8Type = 0x3,
    WChar16Type = 0x4,
    SignedInteger16Type = 0x5,
    UnsignedInteger16Type = 0x6,
    SignedInteger32Type = 0x7,
    UnsignedInteger32Type = 0x8,
    SignedInteger64Type = 0x9,
    UnsignedInteger64Type = 0xA,
    RangedSignedInteger32Type = 0xB,
    RangedUnsignedInteger32Type = 0xC,
    Float32Type = 0xD,
    Float64Type = 0xE,
    RangedFloat32Type = 0xF,
    SignedChar8StringType = 0x10,
    UnsignedChar8StringType = 0x11,
    MbStringType = 0x12,
    BlobType = 0x13,
    NanType = 0x14,
    FullType = 0x15,
    MaxType = 0x20,
}

#[derive(Debug, Copy, Clone)]
pub struct BufferDataType {
    pub primitive_type: BdDataType,
    pub is_array: bool,
}

#[derive(Debug, Snafu)]
#[snafu(display("The value {value} cannot be represented as a BdDataType."))]
struct IllegalDataTypeError {
    value: u8,
}

const ARRAY_TYPE_OFFSET: u8 = 100;

impl BufferDataType {
    pub fn no_array(primitive_type: BdDataType) -> BufferDataType {
        BufferDataType {
            primitive_type,
            is_array: false,
        }
    }

    pub fn array(primitive_type: BdDataType) -> BufferDataType {
        BufferDataType {
            primitive_type,
            is_array: true,
        }
    }

    pub fn eq_non_array(&self, primitive_type: BdDataType) -> bool {
        !self.is_array && self.primitive_type == primitive_type
    }

    pub fn eq_array(&self, primitive_type: BdDataType) -> bool {
        self.is_array && self.primitive_type == primitive_type
    }

    pub fn from_value(value: u8) -> Result<Self, Box<dyn Error>> {
        let is_array = value >= ARRAY_TYPE_OFFSET;
        let non_array_value = if is_array {
            value - ARRAY_TYPE_OFFSET
        } else {
            value
        };
        let primitive_type =
            BdDataType::from_u8(non_array_value).with_context(|| IllegalDataTypeSnafu { value })?;

        Ok(BufferDataType {
            primitive_type,
            is_array,
        })
    }

    pub fn to_value(&self) -> u8 {
        let value = self.primitive_type.to_u8().unwrap();
        if self.is_array {
            value + ARRAY_TYPE_OFFSET
        } else {
            value
        }
    }
}
