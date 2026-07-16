use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum StatsInfoWriteType {
    Replace = 0x0,
    Add = 0x1,
    Max = 0x2,
    Min = 0x3,
    ReplaceWhenRatingIncrease = 0x4,
    AddWhenRatingIncrease = 0x5,
    MaxWhenRatingIncrease = 0x6,
    MinWhenRatingIncrease = 0x7,
}

pub struct StatsInfoResultIn {
    pub leaderboard_id: u32,
    pub entity_id: u64,
    pub write_type: StatsInfoWriteType,
    pub rating: i64,
}

pub struct StatsInfoResultOut {
    pub entity_id: u64,
    pub rating: i64,
    pub rank: u64,
    pub entity_name: String,
    pub seconds_since_update: u32,
}

impl BdSerialize for StatsInfoResultOut {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_u64(self.entity_id)?;
        writer.write_i64(self.rating)?;
        writer.write_u64(self.rank)?;
        writer.write_str(self.entity_name.as_str())?;
        writer.write_u32(self.seconds_since_update)?;

        Ok(())
    }
}
