use crate::lobby::response::lsg_reply::LsgServiceTaskReply;
use crate::messaging::bd_writer::BdWriter;
use crate::messaging::BdErrorCode;
use num_traits::ToPrimitive;
use std::error::Error;

pub struct BandwidthTestRejected {
    reason: BdErrorCode,
}

impl BandwidthTestRejected {
    pub fn with_reason(reason: BdErrorCode) -> BandwidthTestRejected {
        BandwidthTestRejected { reason }
    }
}

impl LsgServiceTaskReply for BandwidthTestRejected {
    fn write_task_reply_data(&self, mut writer: BdWriter) -> Result<(), Box<dyn Error>> {
        // Test rejected
        writer.write_bool(true)?;

        // Rejected reason
        writer.write_u16(self.reason.to_u16().unwrap())?;

        Ok(())
    }
}
