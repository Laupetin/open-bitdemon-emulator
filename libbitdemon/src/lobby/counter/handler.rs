use crate::lobby::counter::result::CounterValueResult;
use crate::lobby::counter::{CounterIncrement, ThreadSafeCounterService};
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_serialization::BdDeserialize;
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::warn;
use num_traits::FromPrimitive;
use std::error::Error;
use std::sync::Arc;

pub struct CounterHandler {
    pub counter_service: Arc<ThreadSafeCounterService>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum CounterTaskId {
    IncrementCounters = 1,
    GetCounterTotals = 2,
}

impl LobbyHandler for CounterHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = CounterTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!(
                "[Session {}] Client called unknown task {task_id_value}",
                session.id
            );
            return Ok(
                TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                    .to_response()?,
            );
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            CounterTaskId::IncrementCounters => {
                self.increment_counters(session, &mut message.reader)
            }
            CounterTaskId::GetCounterTotals => {
                self.get_counter_totals(session, &mut message.reader)
            }
        }
    }
}

impl CounterHandler {
    pub fn new(counter_service: Arc<ThreadSafeCounterService>) -> CounterHandler {
        CounterHandler { counter_service }
    }

    fn increment_counters(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let mut increments = Vec::new();

        while let Ok(counter_value) = CounterValueResult::deserialize(reader) {
            if counter_value.counter_id > 0 {
                increments.push(CounterIncrement {
                    counter_id: counter_value.counter_id,
                    counter_increment: counter_value.counter_value,
                });
            }
        }

        self.counter_service
            .increment_counters(session, increments)?;

        Ok(
            TaskReply::with_only_error_code(BdErrorCode::NoError, CounterTaskId::IncrementCounters)
                .to_response()?,
        )
    }

    fn get_counter_totals(
        &self,
        session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let mut counter_ids = Vec::new();

        while reader.next_is_u32().unwrap_or(false) {
            counter_ids.push(reader.read_u32()?);
        }

        self.counter_service
            .get_counter_totals(session, counter_ids)?;

        Ok(
            TaskReply::with_only_error_code(BdErrorCode::NoError, CounterTaskId::GetCounterTotals)
                .to_response()?,
        )
    }
}
