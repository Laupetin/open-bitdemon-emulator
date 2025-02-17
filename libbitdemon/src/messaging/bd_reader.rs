use crate::messaging::bd_data_type::{BdDataType, BufferDataType};
use crate::messaging::StreamMode;
use byteorder::{LittleEndian, ReadBytesExt};
use snafu::{ensure, Snafu};
use std::cmp::min;
use std::error::Error;
use std::io::{BufRead, Cursor, Read};

#[derive(Debug, Snafu)]
enum BdReaderError {
    #[snafu(display(
        "Expected type {expected_type:?} but got type {actual_type:?} when reading from bdBuffer."
    ))]
    UnexpectedDataType {
        expected_type: BufferDataType,
        actual_type: BufferDataType,
    },
    #[snafu(display("Expected mode {expected_mode:?} but is in mode {actual_mode:?}."))]
    Mode {
        expected_mode: StreamMode,
        actual_mode: StreamMode,
    },
    #[snafu(display("The message terminated unexpectedly."))]
    UnexpectedEndOfMessage,
}

pub struct BdReader {
    cursor: Cursor<Vec<u8>>,
    bit_offset: usize,
    last_byte: u8,
    has_data_type_cached: bool,
    cached_data_type: BufferDataType,
    mode: StreamMode,
    type_checked: bool,
}

impl BdReader {
    pub fn new(buf: Vec<u8>) -> Self {
        BdReader {
            cursor: Cursor::new(buf),
            bit_offset: 8,
            last_byte: 0,
            has_data_type_cached: false,
            cached_data_type: BufferDataType::no_array(BdDataType::NoType),
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

    pub fn read_bits(&mut self, buf: &mut [u8], count: usize) -> Result<(), Box<dyn Error>> {
        debug_assert!(buf.len() * 8 >= count, "Buffer does not fit");

        ensure!(
            self.mode == StreamMode::BitMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::BitMode
            }
        );

        if count == 0 {
            return Ok(());
        }

        let mut bits_left = count;
        let mut dest_offset = 0usize;

        while bits_left > 0 {
            let in_byte = self.last_byte;

            let mut out_byte: u8;
            let max_read_bits: usize;

            // Check if we need a second byte
            if bits_left > 8 - self.bit_offset {
                let in_byte2 = self.cursor.read_u8()?;
                let in_byte_shifted = if self.bit_offset < 8 {
                    in_byte >> self.bit_offset
                } else {
                    0
                };
                out_byte = in_byte_shifted | (in_byte2 << (8 - self.bit_offset));
                self.last_byte = in_byte2;
                max_read_bits = 8;
            } else {
                out_byte = in_byte >> self.bit_offset;
                max_read_bits = 8 - self.bit_offset;
            }

            if bits_left >= 8 {
                bits_left -= max_read_bits;
            } else {
                let read_bits = min(bits_left, max_read_bits);
                self.bit_offset += read_bits;
                if self.bit_offset > 8 {
                    self.bit_offset -= 8;
                }

                out_byte &= 0xFF >> (8 - read_bits);
                bits_left -= read_bits;
            }

            buf[dest_offset] = out_byte;
            dest_offset += 1;
        }

        Ok(())
    }

    pub fn read_bytes(&mut self, buffer: &mut [u8]) -> Result<(), Box<dyn Error>> {
        if self.mode == StreamMode::BitMode {
            return self.read_bits(buffer, buffer.len() * 8);
        }

        ensure!(
            self.cursor.read(buffer)? == buffer.len(),
            UnexpectedEndOfMessageSnafu {}
        );

        Ok(())
    }

    pub fn read_type_checked_bit(&mut self) -> Result<(), Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::BitMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::BitMode
            }
        );

        let mut temp_buffer = [0u8];
        self.read_bits(&mut temp_buffer, 1)?;

        self.type_checked = temp_buffer[0] > 0;

        Ok(())
    }

    fn read_data_type(&mut self) -> Result<BufferDataType, Box<dyn Error>> {
        if self.has_data_type_cached {
            self.has_data_type_cached = false;
            return Ok(self.cached_data_type);
        }

        if self.mode != StreamMode::BitMode {
            return BufferDataType::from_value(self.cursor.read_u8()?);
        }

        let mut temp_buffer = [0u8];
        self.read_bits(&mut temp_buffer, 5)?;

        BufferDataType::from_value(temp_buffer[0])
    }

    fn next_data_type(&mut self) -> Result<BufferDataType, Box<dyn Error>> {
        if !self.type_checked {
            return Ok(BufferDataType::no_array(BdDataType::NoType));
        }

        if self.has_data_type_cached {
            return Ok(self.cached_data_type);
        }

        self.cached_data_type = self.read_data_type()?;
        self.has_data_type_cached = true;

        Ok(self.cached_data_type)
    }

    pub fn next_is_bool(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self.next_data_type()?.eq_non_array(BdDataType::BoolType))
    }

    pub fn next_is_i8(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .next_data_type()?
            .eq_non_array(BdDataType::SignedChar8Type))
    }

    pub fn next_is_u8(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .next_data_type()?
            .eq_non_array(BdDataType::UnsignedChar8Type))
    }

    pub fn next_is_i16(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .next_data_type()?
            .eq_non_array(BdDataType::SignedInteger16Type))
    }

    pub fn next_is_u16(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .next_data_type()?
            .eq_non_array(BdDataType::UnsignedInteger16Type))
    }

    pub fn next_is_i32(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .next_data_type()?
            .eq_non_array(BdDataType::SignedInteger32Type))
    }

    pub fn next_is_u32(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .next_data_type()?
            .eq_non_array(BdDataType::UnsignedInteger32Type))
    }

    pub fn next_is_i64(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .next_data_type()?
            .eq_non_array(BdDataType::SignedInteger64Type))
    }

    pub fn next_is_u64(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .next_data_type()?
            .eq_non_array(BdDataType::UnsignedInteger64Type))
    }

    pub fn next_is_f32(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self.next_data_type()?.eq_non_array(BdDataType::Float32Type))
    }

    pub fn next_is_f64(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self.next_data_type()?.eq_non_array(BdDataType::Float64Type))
    }

    pub fn next_is_str(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .next_data_type()?
            .eq_non_array(BdDataType::SignedChar8StringType))
    }

    pub fn next_is_blob(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self.next_data_type()?.eq_non_array(BdDataType::BlobType))
    }

    fn read_array_num_elements(&mut self) -> Result<usize, Box<dyn Error>> {
        // Always type checked
        let total_size_type = self.read_data_type()?;
        ensure!(
            total_size_type.eq_non_array(BdDataType::UnsignedInteger32Type),
            UnexpectedDataTypeSnafu {
                actual_type: total_size_type,
                expected_type: BufferDataType::no_array(BdDataType::UnsignedInteger32Type)
            }
        );

        // Clients also just ignore this
        let _total_size = self.cursor.read_u32::<LittleEndian>()?;

        // This however is never type checked
        let num_elements = self.cursor.read_u32::<LittleEndian>()?;

        Ok(num_elements as usize)
    }

    pub fn read_bool(&mut self) -> Result<bool, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::BoolType),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::BoolType)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_u8()? > 0);
        }

        let mut temp_buffer = [0u8];
        self.read_bits(&mut temp_buffer, 1)?;

        Ok(temp_buffer[0] > 0)
    }

    pub fn read_i8(&mut self) -> Result<i8, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::SignedChar8Type),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::SignedChar8Type)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_i8()?);
        }

        let mut temp_buffer = [0u8];
        self.read_bits(&mut temp_buffer, i8::BITS as usize)?;

        Ok(i8::from_le_bytes(temp_buffer))
    }

    pub fn read_u8(&mut self) -> Result<u8, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::UnsignedChar8Type),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::UnsignedChar8Type)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_u8()?);
        }

        let mut temp_buffer = [0u8];
        self.read_bits(&mut temp_buffer, u8::BITS as usize)?;

        Ok(u8::from_le_bytes(temp_buffer))
    }

    pub fn read_i16(&mut self) -> Result<i16, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::SignedInteger16Type),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::SignedInteger16Type)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_i16::<LittleEndian>()?);
        }

        let mut temp_buffer = [0u8, 0u8];
        self.read_bits(&mut temp_buffer, i16::BITS as usize)?;

        Ok(i16::from_le_bytes(temp_buffer))
    }

    pub fn read_u16(&mut self) -> Result<u16, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::UnsignedInteger16Type),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::UnsignedInteger16Type)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_u16::<LittleEndian>()?);
        }

        let mut temp_buffer = [0u8, 0u8];
        self.read_bits(&mut temp_buffer, u16::BITS as usize)?;

        Ok(u16::from_le_bytes(temp_buffer))
    }

    pub fn read_i32(&mut self) -> Result<i32, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::SignedInteger32Type),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::SignedInteger32Type)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_i32::<LittleEndian>()?);
        }

        let mut temp_buffer = [0u8, 0u8, 0u8, 0u8];
        self.read_bits(&mut temp_buffer, i32::BITS as usize)?;

        Ok(i32::from_le_bytes(temp_buffer))
    }

    pub fn read_u32(&mut self) -> Result<u32, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::UnsignedInteger32Type),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::UnsignedInteger32Type)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_u32::<LittleEndian>()?);
        }

        let mut temp_buffer = [0u8, 0u8, 0u8, 0u8];
        self.read_bits(&mut temp_buffer, u32::BITS as usize)?;

        Ok(u32::from_le_bytes(temp_buffer))
    }

    pub fn read_i64(&mut self) -> Result<i64, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::SignedInteger64Type),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::SignedInteger64Type)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_i64::<LittleEndian>()?);
        }

        let mut temp_buffer = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        self.read_bits(&mut temp_buffer, i64::BITS as usize)?;

        Ok(i64::from_le_bytes(temp_buffer))
    }

    pub fn read_u64(&mut self) -> Result<u64, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::UnsignedInteger64Type),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::UnsignedInteger64Type)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_u64::<LittleEndian>()?);
        }

        let mut temp_buffer = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        self.read_bits(&mut temp_buffer, u64::BITS as usize)?;

        Ok(u64::from_le_bytes(temp_buffer))
    }

    pub fn read_f32(&mut self) -> Result<f32, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::Float32Type),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::Float32Type)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_f32::<LittleEndian>()?);
        }

        let mut temp_buffer = [0u8, 0u8, 0u8, 0u8];
        self.read_bits(&mut temp_buffer, 32)?;

        Ok(f32::from_le_bytes(temp_buffer))
    }

    pub fn read_f64(&mut self) -> Result<f64, Box<dyn Error>> {
        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::Float64Type),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::Float64Type)
                }
            );
        }

        if self.mode == StreamMode::ByteMode {
            return Ok(self.cursor.read_f64::<LittleEndian>()?);
        }

        let mut temp_buffer = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        self.read_bits(&mut temp_buffer, 64)?;

        Ok(f64::from_le_bytes(temp_buffer))
    }

    pub fn read_str(&mut self) -> Result<String, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::SignedChar8StringType),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::SignedChar8StringType)
                }
            );
        }

        let mut buf = Vec::new();
        self.cursor.read_until(0u8, &mut buf)?;
        if !buf.is_empty() {
            // Remove the 0 byte
            buf.remove(buf.len() - 1);
        }

        Ok(String::from_utf8(buf)?)
    }

    pub fn read_i8_array(&mut self) -> Result<Vec<i8>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::SignedChar8Type),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::SignedChar8Type)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            result.push(self.cursor.read_i8()?);
        }

        Ok(result)
    }

    pub fn read_u8_array(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::UnsignedChar8Type),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::UnsignedChar8Type)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            result.push(self.cursor.read_u8()?);
        }

        Ok(result)
    }

    pub fn read_i16_array(&mut self) -> Result<Vec<i16>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::SignedInteger16Type),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::SignedInteger16Type)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            result.push(self.cursor.read_i16::<LittleEndian>()?);
        }

        Ok(result)
    }

    pub fn read_u16_array(&mut self) -> Result<Vec<u16>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::UnsignedInteger16Type),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::UnsignedInteger16Type)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            result.push(self.cursor.read_u16::<LittleEndian>()?);
        }

        Ok(result)
    }

    pub fn read_i32_array(&mut self) -> Result<Vec<i32>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::SignedInteger32Type),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::SignedInteger32Type)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            result.push(self.cursor.read_i32::<LittleEndian>()?);
        }

        Ok(result)
    }

    pub fn read_u32_array(&mut self) -> Result<Vec<u32>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::UnsignedInteger32Type),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::UnsignedInteger32Type)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            result.push(self.cursor.read_u32::<LittleEndian>()?);
        }

        Ok(result)
    }

    pub fn read_i64_array(&mut self) -> Result<Vec<i64>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::SignedInteger64Type),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::SignedInteger64Type)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            result.push(self.cursor.read_i64::<LittleEndian>()?);
        }

        Ok(result)
    }

    pub fn read_u64_array(&mut self) -> Result<Vec<u64>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::UnsignedInteger64Type),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::UnsignedInteger64Type)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            result.push(self.cursor.read_u64::<LittleEndian>()?);
        }

        Ok(result)
    }

    pub fn read_f32_array(&mut self) -> Result<Vec<f32>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::Float32Type),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::Float32Type)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            result.push(self.cursor.read_f32::<LittleEndian>()?);
        }

        Ok(result)
    }

    pub fn read_f64_array(&mut self) -> Result<Vec<f64>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::Float64Type),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::Float64Type)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            result.push(self.cursor.read_f64::<LittleEndian>()?);
        }

        Ok(result)
    }

    pub fn read_str_array(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        // Arrays are always type checked
        let actual_type = self.read_data_type()?;
        ensure!(
            actual_type.eq_array(BdDataType::SignedChar8StringType),
            UnexpectedDataTypeSnafu {
                actual_type,
                expected_type: BufferDataType::array(BdDataType::SignedChar8StringType)
            }
        );

        let num_elements = self.read_array_num_elements()?;
        let mut result = Vec::with_capacity(num_elements);

        for _ in 0..num_elements {
            let mut buf = Vec::new();
            self.cursor.read_until(0u8, &mut buf)?;
            if !buf.is_empty() {
                // Remove the 0 byte
                buf.remove(buf.len() - 1);
            }

            result.push(String::from_utf8(buf)?);
        }

        Ok(result)
    }

    pub fn read_blob(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        ensure!(
            self.mode == StreamMode::ByteMode,
            ModeSnafu {
                actual_mode: self.mode,
                expected_mode: StreamMode::ByteMode
            }
        );

        if self.type_checked {
            let actual_type = self.read_data_type()?;
            ensure!(
                actual_type.eq_non_array(BdDataType::BlobType),
                UnexpectedDataTypeSnafu {
                    actual_type,
                    expected_type: BufferDataType::no_array(BdDataType::BlobType)
                }
            );
        }

        let blob_size = self.read_u32()? as usize;
        let mut blob = vec![0; blob_size];
        ensure!(
            self.cursor.read(&mut blob[0..blob_size])? == blob_size,
            UnexpectedEndOfMessageSnafu {}
        );

        Ok(blob)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_can_read_bits() {
        let mut reader = BdReader::new(vec![0xA5, 0x00]);
        reader.set_mode(StreamMode::BitMode);

        let mut buf = vec![0u8];

        reader.read_bits(buf.as_mut_slice(), 4).unwrap();
        assert_eq!(buf[0], 5);
    }

    #[test]
    fn ensure_can_read_bits_and_continue() {
        let mut reader = BdReader::new(vec![0xC5, 0x00]);
        reader.set_mode(StreamMode::BitMode);

        let mut buf = vec![0u8];

        reader.read_bits(buf.as_mut_slice(), 4).unwrap();
        assert_eq!(0x05, buf[0]);

        reader.read_bits(buf.as_mut_slice(), 4).unwrap();
        assert_eq!(0x0C, buf[0]);
    }

    #[test]
    fn ensure_can_read_bits_over_two_bytes() {
        let mut reader = BdReader::new(vec![0xE5, 0x4B]);
        reader.set_mode(StreamMode::BitMode);

        let mut buf = vec![0u8];

        reader.read_bits(buf.as_mut_slice(), 6).unwrap();
        assert_eq!(0x25, buf[0]);

        reader.read_bits(buf.as_mut_slice(), 6).unwrap();
        assert_eq!(0x2F, buf[0]);
    }

    #[test]
    fn ensure_can_read_bits_multiple_bytes() {
        let mut reader = BdReader::new(vec![0xE5, 0x4B, 0xC1]);
        reader.set_mode(StreamMode::BitMode);

        let mut buf = vec![0u8, 0u8, 0u8];

        reader.read_bits(buf.as_mut_slice(), 2).unwrap();
        assert_eq!(0x01, buf[0]);

        reader.read_bits(buf.as_mut_slice(), 14).unwrap();
        assert_eq!(0xF9, buf[0]);
        assert_eq!(0x12, buf[1]);

        reader.read_bits(buf.as_mut_slice(), 8).unwrap();
        assert_eq!(0xC1, buf[0]);
    }

    #[test]
    fn ensure_can_read_bits_multiple_times_in_one_byte() {
        let mut reader = BdReader::new(vec![0xE5]);
        reader.set_mode(StreamMode::BitMode);

        let mut buf = vec![0u8];

        reader.read_bits(buf.as_mut_slice(), 2).unwrap();
        assert_eq!(0x01, buf[0]);

        reader.read_bits(buf.as_mut_slice(), 4).unwrap();
        assert_eq!(0x09, buf[0]);

        reader.read_bits(buf.as_mut_slice(), 2).unwrap();
        assert_eq!(0x03, buf[0]);
    }

    #[test]
    fn ensure_reads_as_many_bits_as_possible() {
        let mut reader = BdReader::new(vec![0xE5]);
        reader.set_mode(StreamMode::BitMode);

        let mut buf = vec![0u8];

        reader.read_bits(buf.as_mut_slice(), 4).unwrap();
        assert_eq!(0x05, buf[0]);

        let maybe_error = reader.read_bits(buf.as_mut_slice(), 5);
        assert!(maybe_error.is_err());
    }

    #[test]
    fn ensure_can_handle_giant_bit_read_count() {
        let mut reader = BdReader::new(vec![0xE5, 0xBB, 0x1F, 0x22]);
        reader.set_mode(StreamMode::BitMode);

        let mut buf = vec![0u8, 0u8, 0u8, 0u8, 0u8];

        reader.read_bits(buf.as_mut_slice(), 32).unwrap();
        assert_eq!(0xE5, buf[0]);
        assert_eq!(0xBB, buf[1]);
        assert_eq!(0x1F, buf[2]);
        assert_eq!(0x22, buf[3]);
        assert_eq!(0x00, buf[4]);
    }

    #[test]
    #[should_panic]
    fn ensure_cannot_read_bits_into_buffer_that_is_too_small() {
        let mut reader = BdReader::new(vec![0xE5, 0xBB, 0x1F, 0x22]);
        reader.set_mode(StreamMode::BitMode);

        let mut buf = vec![0u8, 0u8, 0u8];
        let res = reader.read_bits(buf.as_mut_slice(), 32);
        assert!(res.is_err())
    }

    #[test]
    fn ensure_read_bit_returns_zero_if_none_should_be_read() {
        let mut reader = BdReader::new(vec![0xE5, 0xBB, 0x1F, 0x22]);
        reader.set_mode(StreamMode::BitMode);

        let mut buf = vec![0u8, 0u8, 0u8];
        reader.read_bits(buf.as_mut_slice(), 0).unwrap();

        assert_eq!(buf[0], 0);
    }

    #[test]
    fn ensure_read_bit_errors_when_in_byte_mode() {
        let mut reader = BdReader::new(vec![0xE5, 0xBB, 0x1F, 0x22]);
        reader.set_mode(StreamMode::ByteMode);

        let mut buf = vec![0u8, 0u8, 0u8];

        assert!(reader.read_bits(buf.as_mut_slice(), 5).is_err());
        assert_eq!(buf[0], 0);
    }

    #[test]
    fn ensure_read_bool_can_handle_single_bits() {
        let mut reader = BdReader::new(vec![0x65]);
        reader.set_mode(StreamMode::BitMode);

        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
    }

    #[test]
    fn ensure_read_bool_can_handle_bytes() {
        let mut reader = BdReader::new(vec![0x00, 0x01, 0x01, 0x00, 0x00, 0x02, 0xFF, 0xBB, 0x00]);
        reader.set_mode(StreamMode::ByteMode);

        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
    }

    #[test]
    fn ensure_read_bool_errors_in_byte_mode_when_out_of_bounds() {
        let mut reader = BdReader::new(vec![0x00, 0x01]);
        reader.set_mode(StreamMode::ByteMode);

        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(reader.read_bool().is_err());
        assert!(reader.read_bool().is_err());
    }

    #[test]
    fn ensure_read_bool_errors_in_bit_mode_when_out_of_bounds() {
        let mut reader = BdReader::new(vec![0x55]);
        reader.set_mode(StreamMode::BitMode);

        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().is_err());
        assert!(reader.read_bool().is_err());
    }

    #[test]
    fn ensure_can_read_bool_with_type_check_in_bit_mode() {
        let mut reader = BdReader::new(vec![0x61, 0xF0]);
        reader.set_mode(StreamMode::BitMode);
        reader.set_type_checked(true);

        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().is_err());
    }

    #[test]
    fn ensure_can_read_bool_with_type_check_in_byte_mode() {
        let mut reader = BdReader::new(vec![0x01, 0x05, 0x01, 0x00]);
        reader.set_mode(StreamMode::ByteMode);
        reader.set_type_checked(true);

        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().is_err());
    }

    #[test]
    fn ensure_errors_when_reading_bool_in_bit_mode_and_type_does_not_match() {
        let mut reader = BdReader::new(vec![0x62, 0xF0]);
        reader.set_mode(StreamMode::BitMode);
        reader.set_type_checked(true);

        assert!(reader.read_bool().is_err());
    }

    #[test]
    fn ensure_throws_exception_when_reading_bool_in_byte_mode_and_type_does_not_match() {
        let mut reader = BdReader::new(vec![0x03, 0x01]);
        reader.set_mode(StreamMode::ByteMode);
        reader.set_type_checked(true);

        assert!(reader.read_bool().is_err());
    }
}
