mod service;

use crate::lobby::counter::service::DwCounterService;
use bitdemon::lobby::counter::CounterHandler;
use bitdemon::lobby::ThreadSafeLobbyHandler;
use std::sync::Arc;

pub fn create_counter_handler() -> Arc<ThreadSafeLobbyHandler> {
    Arc::new(CounterHandler::new(Arc::new(DwCounterService::new())))
}
