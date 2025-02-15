use crate::lobby::storage::service::{FileVisibility, StorageFileInfo};
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_serialization::{BdDeserialize, BdSerialize};
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

impl BdSerialize for StorageFileInfo {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u32(self.file_size as u32)?;
        writer.write_u64(self.id)?;
        writer.write_u32((self.created % (u32::MAX as i64)) as u32)?;
        writer.write_bool(self.visibility == FileVisibility::VisiblePrivate)?;
        writer.write_u64(self.owner_id)?;
        writer.write_str(self.filename.as_str())?;

        Ok(())
    }
}

pub struct FileDataResult {
    pub data: Vec<u8>,
}

impl BdDeserialize for FileDataResult {
    fn deserialize(reader: &mut BdReader) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let data = reader.read_blob()?;

        Ok(FileDataResult { data })
    }
}

impl BdSerialize for FileDataResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_blob(self.data.as_slice())
    }
}
