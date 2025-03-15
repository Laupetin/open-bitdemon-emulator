use crate::lobby::key_archive::result::KeyValuePairWriteResult;
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

pub struct KeyArchiveHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum KeyArchiveTaskId {
    Write = 1,
    Read = 2,
    ReadAll = 3,
    ReadMultipleEntityIds = 4,
}

impl LobbyHandler for KeyArchiveHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = KeyArchiveTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                .to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            KeyArchiveTaskId::Write => Self::write(session, &mut message.reader),
            KeyArchiveTaskId::Read => Self::read(session, &mut message.reader),
            KeyArchiveTaskId::ReadAll => Self::read_all(session, &mut message.reader),
            KeyArchiveTaskId::ReadMultipleEntityIds => {
                Self::read_multiple_entity_ids(session, &mut message.reader)
            }
        }
    }
}

impl Default for KeyArchiveHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyArchiveHandler {
    pub fn new() -> KeyArchiveHandler {
        KeyArchiveHandler {}
    }

    fn write(
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let entity_id = reader.read_u64()?;

        if reader.next_is_u16().unwrap_or(false) {
            let category_id = reader.read_u16()?;
            let mut kvps = Vec::new();

            while let Ok(kvp) = KeyValuePairWriteResult::deserialize(reader) {
                kvps.push(kvp);
            }

            // TODO: Call service

            info!("Writing key value pairs for {entity_id} of category {category_id} with kvps: {kvps:?}");
        }

        TaskReply::with_only_error_code(BdErrorCode::NoError, KeyArchiveTaskId::Write).to_response()
    }

    fn read(_session: &mut BdSession, reader: &mut BdReader) -> Result<BdResponse, Box<dyn Error>> {
        let entity_id = reader.read_u64()?;

        if reader.next_is_u16().unwrap_or(false) {
            let category_id = reader.read_u16()?;
            let read_dedicated = reader.read_bool()?;
            let mut indices = Vec::new();

            while reader.next_is_u16().unwrap_or(false) {
                indices.push(reader.read_u16()?);
            }

            // TODO: Call service

            info!(
                "Requesting key value pairs for {entity_id} of category {category_id} (dedicated={read_dedicated}) with indices: {indices:?}");
        }

        TaskReply::with_only_error_code(BdErrorCode::NoError, KeyArchiveTaskId::Read).to_response()
    }

    fn read_all(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        // TODO
        TaskReply::with_only_error_code(BdErrorCode::NoError, KeyArchiveTaskId::ReadAll)
            .to_response()
    }

    fn read_multiple_entity_ids(
        _session: &mut BdSession,
        _reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        // TODO
        TaskReply::with_only_error_code(BdErrorCode::NoError, KeyArchiveTaskId::ReadAll)
            .to_response()
    }
}
