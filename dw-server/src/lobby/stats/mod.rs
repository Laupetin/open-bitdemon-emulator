use bitdemon::lobby::ThreadSafeLobbyHandler;
use bitdemon::lobby::stats::StatsHandler;
use std::sync::Arc;

pub fn create_stats_handler() -> Arc<ThreadSafeLobbyHandler> {
    Arc::new(StatsHandler::new())
}
