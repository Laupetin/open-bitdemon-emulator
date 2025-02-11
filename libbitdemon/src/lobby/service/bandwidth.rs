use crate::lobby::response::lsg_reply::{LsgResponseCreator, LsgServiceTaskReply};
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_writer::BdWriter;
use crate::messaging::BdErrorCode;
use crate::messaging::BdErrorCode::NoError;
use crate::networking::bd_session::BdSession;
use log::{debug, warn};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::error::Error;

pub struct BandwidthHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum BandwidthTaskId {
    BandwidthTask = 1,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum BandwidthTestType {
    UploadTest = 0,
    UploadDownloadTest = 1,
}

impl LobbyHandler for BandwidthHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        message.reader.set_type_checked(false);

        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = BandwidthTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!(
                "[Session {}] Client called unknown task {task_id_value}",
                session.id
            );
            return Ok(TaskReply::with_only_error_code(NoError).to_response()?);
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            BandwidthTaskId::BandwidthTask => {
                Self::handle_bandwidth_task(session, &mut message.reader)
            }
        }
    }
}

impl BandwidthHandler {
    pub fn new() -> BandwidthHandler {
        BandwidthHandler {}
    }

    fn handle_bandwidth_task(
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let test_type_value = reader.read_u8()?;
        match BandwidthTestType::from_u8(test_type_value) {
            Some(test_type) => {
                debug!(
                    "[Session {}] Client requested bandwidth test type={test_type:?}",
                    session.id
                );
            }
            None => {
                warn!(
                    "[Session {}] Client requested unknown bandwidth test type={test_type_value}",
                    session.id
                )
            }
        }

        // Bandwidth tests are not supported
        Ok(BandwidthTestRejected::with_reason(BdErrorCode::ServiceNotAvailable).to_response()?)
    }
}

struct BandwidthTestRejected {
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
