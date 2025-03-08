use crate::lobby::content_streaming::{StreamInfo, StreamUrl};
use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

pub struct FileIdResult {
    pub id: u64,
}

impl BdSerialize for StreamInfo {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u64(self.id)?;
        writer.write_u32((self.created % u32::MAX as i64) as u32)?;
        writer.write_u32((self.modified % u32::MAX as i64) as u32)?;
        writer.write_u32((self.stream_size % u32::MAX as u64) as u32)?;
        writer.write_u64(self.owner_id)?;
        writer.write_str(self.owner_name.as_str())?;
        writer.write_u16(self.slot)?;
        writer.write_str(self.filename.as_str())?;
        writer.write_str(self.url.as_str())?;
        writer.write_u16(self.category)?;
        writer.write_blob(self.metadata.as_slice())?;
        writer.write_u32((self.summary_file_size % u32::MAX as u64) as u32)?;

        let mut tags = Vec::with_capacity(self.tags.len() * 2);
        for tag in self.tags.as_slice() {
            tags.push(tag.primary);
            tags.push(tag.secondary);
        }

        writer.write_u64_array(tags.as_slice())?;
        writer.write_u32(self.num_copies_made)?;
        writer.write_u64(self.origin_id)
    }
}

impl BdSerialize for StreamUrl {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_str(self.url.as_str())?;
        writer.write_u16(self.server_type)?;
        writer.write_str(self.server_index.as_str())?;
        writer.write_u64(self.stream_id)
    }
}

impl BdSerialize for FileIdResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u64(self.id)
    }
}
