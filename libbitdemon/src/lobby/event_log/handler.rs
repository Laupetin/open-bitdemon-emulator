use crate::lobby::event_log::result::EventInfo;
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_serialization::BdDeserialize;
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::{info, warn};
use num_traits::FromPrimitive;
use std::error::Error;

pub struct EventLogHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
#[allow(clippy::enum_variant_names)]
enum EventLogTaskId {
    RecordEvent = 1,
    RecordEventBin = 2,
    RecordEvents = 3,
    RecordEventsMixed = 5,
}

impl LobbyHandler for EventLogHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = EventLogTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                .to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            EventLogTaskId::RecordEvent => Self::record_event(session, &mut message.reader),
            EventLogTaskId::RecordEventBin => Self::record_event_bin(session, &mut message.reader),
            EventLogTaskId::RecordEvents => Self::record_events(session, &mut message.reader),
            EventLogTaskId::RecordEventsMixed => {
                Self::record_events_mixed(session, &mut message.reader)
            }
        }
    }
}

impl Default for EventLogHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLogHandler {
    pub fn new() -> EventLogHandler {
        EventLogHandler {}
    }

    fn record_event(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let string_event = reader.read_str()?;
        let category_id = reader.read_u32()?;

        info!("Recording event category={category_id} event={string_event}");

        TaskReply::with_only_error_code(BdErrorCode::NoError, EventLogTaskId::RecordEvent)
            .to_response()
    }

    fn record_event_bin(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let binary_data = reader.read_blob()?;
        let category_id = reader.read_u32()?;

        info!(
            "Recording binary event category={category_id} data_len={}",
            binary_data.len()
        );

        TaskReply::with_only_error_code(BdErrorCode::NoError, EventLogTaskId::RecordEventBin)
            .to_response()
    }

    fn record_events(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let category_id = reader.read_u32()?;
        let event_count = reader.read_u32()?;

        for _ in 0..event_count {
            let string_event = reader.read_str()?;
            info!("Recording event category={category_id} event={string_event}");
        }

        TaskReply::with_only_error_code(BdErrorCode::NoError, EventLogTaskId::RecordEvents)
            .to_response()
    }

    fn record_events_mixed(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let event_count = reader.read_u32()?;

        for _ in 0..event_count {
            let event_info = EventInfo::deserialize(reader)?;
            if let Some(binary_data) = event_info.binary_data {
                info!(
                    "Recording binary event category={} data_len={}",
                    event_info.category_id,
                    binary_data.len()
                );
            } else if let Some(string_data) = event_info.string_data {
                info!(
                    "Recording event category={} event={}",
                    event_info.category_id, string_data
                );
            }
        }

        TaskReply::with_only_error_code(BdErrorCode::NoError, EventLogTaskId::RecordEvents)
            .to_response()
    }
}
