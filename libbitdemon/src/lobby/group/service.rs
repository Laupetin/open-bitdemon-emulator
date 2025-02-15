use crate::networking::bd_session::BdSession;
use std::error::Error;

pub type ThreadSafeGroupService = dyn GroupService + Sync + Send;

/// Implements domain logic concerning groups.
pub trait GroupService {
    /// Increments stored counters by the specified amounts.
    fn get_group_counts(
        &self,
        session: &BdSession,
        groups: &[u32],
    ) -> Result<Vec<u64>, Box<dyn Error>>;

    /// Adds the current session to the specified groups
    fn set_groups(&self, session: &BdSession, groups: &[u32]) -> Result<(), Box<dyn Error>>;
}
