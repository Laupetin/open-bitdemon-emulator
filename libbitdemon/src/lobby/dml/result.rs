use crate::messaging::bd_serialization::BdSerialize;
use crate::messaging::bd_writer::BdWriter;
use std::error::Error;

pub struct DmlInfoResult {
    pub country_code: String,
    pub country: String,
    pub region: String,
    pub city: String,
    pub latitude: f32,
    pub longitude: f32,
}

pub struct DmlHierarchicalInfoResult {
    pub base: DmlInfoResult,
    pub tier0: u32,
    pub tier1: u32,
    pub tier2: u32,
    pub tier3: u32,
}

impl BdSerialize for DmlInfoResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        writer.write_str(self.country_code.as_str())?;
        writer.write_str(self.country.as_str())?;
        writer.write_str(self.region.as_str())?;
        writer.write_str(self.city.as_str())?;
        writer.write_f32(self.latitude)?;
        writer.write_f32(self.longitude)?;

        Ok(())
    }
}

impl BdSerialize for DmlHierarchicalInfoResult {
    fn serialize(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        self.base.serialize(writer)?;
        writer.write_u32(self.tier0)?;
        writer.write_u32(self.tier1)?;
        writer.write_u32(self.tier2)?;
        writer.write_u32(self.tier3)?;

        Ok(())
    }
}
