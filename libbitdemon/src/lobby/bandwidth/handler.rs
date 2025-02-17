use crate::lobby::bandwidth::result::BandwidthTestRejected;
use crate::lobby::response::lsg_reply::LsgResponseCreator;
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::BdErrorCode;
use crate::messaging::BdErrorCode::NoError;
use crate::networking::bd_session::BdSession;
use log::{debug, warn};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
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
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(NoError, task_id_value).to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            BandwidthTaskId::BandwidthTask => {
                Self::handle_bandwidth_task(session, &mut message.reader)
            }
        }
    }
}

impl Default for BandwidthHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl BandwidthHandler {
    pub fn new() -> BandwidthHandler {
        BandwidthHandler {}
    }

    fn handle_bandwidth_task(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let test_type_value = reader.read_u8()?;
        match BandwidthTestType::from_u8(test_type_value) {
            Some(test_type) => {
                debug!("Client requested bandwidth test type={test_type:?}");
            }
            None => {
                warn!("Client requested unknown bandwidth test type={test_type_value}")
            }
        }

        // Bandwidth tests are not supported
        BandwidthTestRejected::with_reason(BdErrorCode::ServiceNotAvailable).to_response()
    }
}
