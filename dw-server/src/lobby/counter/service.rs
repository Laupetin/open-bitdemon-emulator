use bitdemon::lobby::counter::{CounterIncrement, CounterService, CounterValue};
use bitdemon::networking::bd_session::BdSession;
use log::info;
use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;

pub struct DwCounterService {
    data: RwLock<HashMap<u32, i64>>,
}

impl CounterService for DwCounterService {
    fn get_counter_totals(
        &self,
        _session: &BdSession,
        counter_ids: Vec<u32>,
    ) -> Result<Vec<CounterValue>, Box<dyn Error>> {
        info!(
            "Retrieving counter totals for {} counters",
            counter_ids.len()
        );

        let mut result = Vec::with_capacity(counter_ids.len());

        let data = self.data.read().unwrap();
        for counter_id in counter_ids {
            let counter_value = data.get(&counter_id).copied().unwrap_or(0);
            result.push(CounterValue {
                counter_id,
                counter_value,
            })
        }

        Ok(result)
    }

    fn increment_counters(
        &self,
        _session: &BdSession,
        increments: Vec<CounterIncrement>,
    ) -> Result<(), Box<dyn Error>> {
        info!(
            "Incrementing counter totals for {} counters",
            increments.len()
        );

        let mut data = self.data.write().unwrap();
        for increment in increments {
            if let Some(existing_value) = data.get_mut(&increment.counter_id) {
                *existing_value += increment.counter_increment;
            } else {
                data.insert(increment.counter_id, increment.counter_increment);
            }
        }

        Ok(())
    }
}

impl DwCounterService {
    pub fn new() -> DwCounterService {
        DwCounterService {
            data: RwLock::new(HashMap::new()),
        }
    }
}
