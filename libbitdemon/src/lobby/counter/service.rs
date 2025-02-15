use crate::networking::bd_session::BdSession;
use std::error::Error;

pub struct CounterIncrement {
    pub counter_id: u32,
    pub counter_increment: i64,
}

pub struct CounterValue {
    pub counter_id: u32,
    pub counter_value: i64,
}

pub type ThreadSafeCounterService = dyn CounterService + Sync + Send;

/// Implements domain logic concerning counters.
pub trait CounterService {
    /// Increments stored counters by the specified amounts.
    fn get_counter_totals(
        &self,
        session: &BdSession,
        counter_ids: Vec<u32>,
    ) -> Result<Vec<CounterValue>, Box<dyn Error>>;

    /// Increments stored counters by the specified amounts.
    fn increment_counters(
        &self,
        session: &BdSession,
        increments: Vec<CounterIncrement>,
    ) -> Result<(), Box<dyn Error>>;
}
