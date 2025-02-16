use crate::messaging::bd_data_type::{BdDataType, BufferDataType};
use crate::messaging::StreamMode;
use byteorder::{LittleEndian, WriteBytesExt};
use snafu::{ensure, Snafu};
use std::error::Error;
use std::io::{Cursor, Write};

#[derive(Debug, Snafu)]
enum BdWriterError {
    #[snafu(display("Expected mode {expected_mode:?} but is in mode {actual_mode:?}."))]
    ModeError {
        expected_mode: StreamMode,
        actual_mode: StreamMode,
    },
}

pub struct BdWriter<'a> {
    cursor: Cursor<&'a mut Vec<u8>>,
    bit_offset: usize,
    last_byte: u8,
    mode: StreamMode,
    type_checked: bool,
}

impl<'a> BdWriter<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        BdWriter {
            cursor: Cursor::new(buf),
            bit_offset: 8,
            last_byte: 0,
            mode: StreamMode::ByteMode,
            type_checked: false,
        }
    }

    pub fn mode(&self) -> StreamMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: StreamMode) {
        self.mode = mode;
    }

    pub fn type_checked(&self) -> bool {
        self.type_checked
    }

    pub fn set_type_checked(&mut self, type_checked: bool) {
        self.type_checked = type_checked;
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        if self.bit_offset >= 8 {
            return Ok(());
        }

        self.cursor.write_u8(self.last_byte)?;
        self.bit_offset = 8;

        Ok(())
    }

    pub fn write_bits(&mut self, buf: &[u8], count: usize) -> Result<(), Box<dyn Error>> {
        debug_assert!(buf.len() * 8 >= count, "Buffer does not fit");

        ensure!(
            self.mode == StreamMode::BitMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::BitMode
            }
        );

        if count <= 0 {
            return Ok(());
        }

        let mut bits_left = count;
        let mut src_offset = 0usize;

        while bits_left > 0 {
            let mut in_bits = 8;
            let mut in_byte = buf[src_offset];
            src_offset += 1;
            if bits_left < 8 {
                in_bits = bits_left;
                in_byte &= 0xFF >> (8 - bits_left);
            }

            if self.bit_offset < 8 {
                self.last_byte |= in_byte << self.bit_offset;

                if self.bit_offset + in_bits > 8 {
                    let used_bits = 8 - self.bit_offset;
                    self.cursor.write_u8(self.last_byte)?;
                    self.bit_offset = (self.bit_offset as i64 + (in_bits as i64 - 8)) as usize;
                    self.last_byte = in_byte >> used_bits;
                } else if self.bit_offset + in_bits == 8 {
                    self.cursor.write_u8(self.last_byte)?;
                    self.last_byte = 0;
                    self.bit_offset = 8;
                } else {
                    self.bit_offset += in_bits;
                }
            } else if in_bits == 8 {
                self.cursor.write_u8(in_byte)?;
            } else {
                self.last_byte = in_byte;
                self.bit_offset = in_bits;
            }

            bits_left -= in_bits;
        }

        Ok(())
    }

    pub fn write_bytes(&mut self, buffer: &[u8]) -> Result<(), Box<dyn Error>> {
        if self.mode == StreamMode::BitMode {
            self.write_bits(buffer, buffer.len() * 8)
        } else {
            self.cursor.write(buffer)?;
            Ok(())
        }
    }

    pub fn write_type_checked_bit(&mut self) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::BitMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::BitMode
            }
        );

        self.write_bits(if self.type_checked { &[0x01] } else { &[0x00] }, 1)?;

        Ok(())
    }

    fn write_data_type(&mut self, buffer_data_type: BufferDataType) -> Result<(), Box<dyn Error>> {
        let value = buffer_data_type.to_value();
        if self.mode == StreamMode::ByteMode {
            self.cursor.write_u8(value)?;
            Ok(())
        } else {
            self.write_bits(&[value], 5)
        }
    }

    fn write_array_num_elements(&mut self, num_elements: usize) -> Result<(), Box<dyn Error>> {
        // Always type checked
        self.write_data_type(BufferDataType::no_array(BdDataType::UnsignedInteger32Type))?;

        // TotalSize: Clients just ignore this
        self.cursor.write_u32::<LittleEndian>(0)?;

        // This however is never type checked
        self.cursor.write_u32::<LittleEndian>(num_elements as u32)?;

        Ok(())
    }

    pub fn write_bool(&mut self, value: bool) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::BoolType))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_u8(if value { 1 } else { 0 })?;
            Ok(())
        } else {
            self.write_bits(if value { &[0x01] } else { &[0x00] }, 1)
        }
    }

    pub fn write_i8(&mut self, value: i8) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::SignedChar8Type))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_i8(value)?;
            Ok(())
        } else {
            self.write_bits(&value.to_le_bytes(), i8::BITS as usize)
        }
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::UnsignedChar8Type))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_u8(value)?;
            Ok(())
        } else {
            self.write_bits(&value.to_le_bytes(), u8::BITS as usize)
        }
    }

    pub fn write_i16(&mut self, value: i16) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::SignedInteger16Type))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_i16::<LittleEndian>(value)?;
            Ok(())
        } else {
            self.write_bits(&value.to_le_bytes(), i16::BITS as usize)
        }
    }

    pub fn write_u16(&mut self, value: u16) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::UnsignedInteger16Type))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_u16::<LittleEndian>(value)?;
            Ok(())
        } else {
            self.write_bits(&value.to_le_bytes(), u16::BITS as usize)
        }
    }

    pub fn write_i32(&mut self, value: i32) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::SignedInteger32Type))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_i32::<LittleEndian>(value)?;
            Ok(())
        } else {
            self.write_bits(&value.to_le_bytes(), i32::BITS as usize)
        }
    }

    pub fn write_u32(&mut self, value: u32) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::UnsignedInteger32Type))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_u32::<LittleEndian>(value)?;
            Ok(())
        } else {
            self.write_bits(&value.to_le_bytes(), u32::BITS as usize)
        }
    }

    pub fn write_i64(&mut self, value: i64) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::SignedInteger64Type))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_i64::<LittleEndian>(value)?;
            Ok(())
        } else {
            self.write_bits(&value.to_le_bytes(), i64::BITS as usize)
        }
    }

    pub fn write_u64(&mut self, value: u64) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::UnsignedInteger64Type))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_u64::<LittleEndian>(value)?;
            Ok(())
        } else {
            self.write_bits(&value.to_le_bytes(), u64::BITS as usize)
        }
    }

    pub fn write_f32(&mut self, value: f32) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::Float32Type))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_f32::<LittleEndian>(value)?;
            Ok(())
        } else {
            self.write_bits(&value.to_le_bytes(), 32)
        }
    }

    pub fn write_f64(&mut self, value: f64) -> Result<(), Box<dyn Error>> {
        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::Float64Type))?;
        }

        if self.mode == StreamMode::ByteMode {
            self.cursor.write_f64::<LittleEndian>(value)?;
            Ok(())
        } else {
            self.write_bits(&value.to_le_bytes(), 64)
        }
    }

    pub fn write_str(&mut self, value: &str) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::SignedChar8StringType))?;
        }

        self.cursor.write(value.as_bytes())?;
        self.cursor.write_u8(0)?;

        Ok(())
    }

    pub fn write_i8_array(&mut self, value: &[i8]) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        self.write_data_type(BufferDataType::array(BdDataType::SignedChar8Type))?;

        self.write_array_num_elements(value.len())?;

        for el in value {
            self.cursor.write_i8(*el)?;
        }

        Ok(())
    }

    pub fn write_u8_array(&mut self, value: &[u8]) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        self.write_data_type(BufferDataType::array(BdDataType::UnsignedChar8Type))?;

        self.write_array_num_elements(value.len())?;

        for el in value {
            self.cursor.write_u8(*el)?;
        }

        Ok(())
    }

    pub fn write_i16_array(&mut self, value: &[i16]) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        self.write_data_type(BufferDataType::array(BdDataType::SignedInteger16Type))?;

        self.write_array_num_elements(value.len())?;

        for el in value {
            self.cursor.write_i16::<LittleEndian>(*el)?;
        }

        Ok(())
    }

    pub fn write_u16_array(&mut self, value: &[u16]) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        self.write_data_type(BufferDataType::array(BdDataType::UnsignedInteger16Type))?;

        self.write_array_num_elements(value.len())?;

        for el in value {
            self.cursor.write_u16::<LittleEndian>(*el)?;
        }

        Ok(())
    }

    pub fn write_i32_array(&mut self, value: &[i32]) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        self.write_data_type(BufferDataType::array(BdDataType::SignedInteger32Type))?;

        self.write_array_num_elements(value.len())?;

        for el in value {
            self.cursor.write_i32::<LittleEndian>(*el)?;
        }

        Ok(())
    }

    pub fn write_u32_array(&mut self, value: &[u32]) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        self.write_data_type(BufferDataType::array(BdDataType::UnsignedInteger32Type))?;

        self.write_array_num_elements(value.len())?;

        for el in value {
            self.cursor.write_u32::<LittleEndian>(*el)?;
        }

        Ok(())
    }

    pub fn write_i64_array(&mut self, value: &[i64]) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        self.write_data_type(BufferDataType::array(BdDataType::SignedInteger64Type))?;

        self.write_array_num_elements(value.len())?;

        for el in value {
            self.cursor.write_i64::<LittleEndian>(*el)?;
        }

        Ok(())
    }

    pub fn write_u64_array(&mut self, value: &[u64]) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        self.write_data_type(BufferDataType::array(BdDataType::UnsignedInteger64Type))?;

        self.write_array_num_elements(value.len())?;

        for el in value {
            self.cursor.write_u64::<LittleEndian>(*el)?;
        }

        Ok(())
    }

    pub fn write_str_array(&mut self, value: &[&str]) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        self.write_data_type(BufferDataType::array(BdDataType::SignedChar8StringType))?;

        self.write_array_num_elements(value.len())?;

        for el in value {
            self.cursor.write(el.as_bytes())?;
            self.cursor.write_u8(0)?;
        }

        Ok(())
    }

    pub fn write_blob(&mut self, value: &[u8]) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        if self.type_checked {
            self.write_data_type(BufferDataType::no_array(BdDataType::BlobType))?;
        }

        self.write_u32(value.len() as u32)?;
        self.cursor.write(value)?;

        Ok(())
    }
}

impl Drop for BdWriter<'_> {
    fn drop(&mut self) {
        self.flush().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_can_write_bits() {
        let mut out = Vec::new();

        {
            let mut writer = BdWriter::new(&mut out);
            writer.set_mode(StreamMode::BitMode);

            writer.write_bits(&[0x55], 6).unwrap();
        }

        assert_eq!(out[0], 0x15);
    }

    #[test]
    fn ensure_can_write_bits_and_continue() {
        let mut out = Vec::new();

        {
            let mut writer = BdWriter::new(&mut out);
            writer.set_mode(StreamMode::BitMode);

            writer.write_bits(&[0x55], 6).unwrap();
            writer.write_bits(&[0xAA], 2).unwrap();
        }

        assert_eq!(out[0], 0x95);
    }

    #[test]
    fn ensure_can_write_exactly_one_byte_in_bits() {
        let mut out = Vec::new();

        {
            let mut writer = BdWriter::new(&mut out);
            writer.set_mode(StreamMode::BitMode);

            writer.write_bits(&[0x4B], 8).unwrap();
        }

        assert_eq!(out[0], 0x4B);
    }

    #[test]
    fn ensure_can_write_over_byte_boundary() {
        let mut out = Vec::new();

        {
            let mut writer = BdWriter::new(&mut out);
            writer.set_mode(StreamMode::BitMode);

            writer.write_bits(&[0x0B], 4).unwrap();
            writer.write_bits(&[0x9D], 8).unwrap();
            writer.write_bits(&[0x0D], 4).unwrap();
        }

        assert_eq!(out[0], 0xDB);
        assert_eq!(out[1], 0xD9);
    }

    #[test]
    fn ensure_can_write_over_byte_boundary_with_less_than_one_byte() {
        let mut out = Vec::new();

        {
            let mut writer = BdWriter::new(&mut out);
            writer.set_mode(StreamMode::BitMode);

            writer.write_bits(&[0x3F], 6).unwrap();
            writer.write_bits(&[0x06], 4).unwrap();
            writer.write_bits(&[0x3F], 6).unwrap();
        }

        assert_eq!(out[0], 0xBF);
        assert_eq!(out[1], 0xFD);
    }

    #[test]
    fn ensure_can_write_multiple_times_in_one_byte() {
        let mut out = Vec::new();

        {
            let mut writer = BdWriter::new(&mut out);
            writer.set_mode(StreamMode::BitMode);

            writer.write_bits(&[0x01], 2).unwrap();
            writer.write_bits(&[0x02], 2).unwrap();
            writer.write_bits(&[0x01], 2).unwrap();
            writer.write_bits(&[0x02], 2).unwrap();
        }

        assert_eq!(out[0], 0x99);
    }

    #[test]
    fn ensure_can_write_u32() {
        let mut out = Vec::new();

        {
            let mut writer = BdWriter::new(&mut out);
            writer.set_mode(StreamMode::BitMode);

            writer.write_u32(0x32).unwrap();
        }

        assert_eq!(out[0], 0x32);
        assert_eq!(out[1], 0);
        assert_eq!(out[2], 0);
        assert_eq!(out[3], 0);
    }

    #[test]
    fn ensure_can_write_u32_with_types() {
        let mut out = Vec::new();

        {
            let mut writer = BdWriter::new(&mut out);
            writer.set_mode(StreamMode::BitMode);
            writer.set_type_checked(true);

            writer.write_u32(0x32).unwrap();
        }

        assert_eq!(out[0], 0x48);
        assert_eq!(out[1], 0x06);
        assert_eq!(out[2], 0);
        assert_eq!(out[3], 0);
        assert_eq!(out[4], 0);
    }
}
