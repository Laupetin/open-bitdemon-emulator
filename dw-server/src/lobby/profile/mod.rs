mod db;
mod service;

use crate::lobby::profile::service::DwProfileService;
use bitdemon::lobby::ThreadSafeLobbyHandler;
use bitdemon::lobby::profile::ProfileHandler;
use std::sync::Arc;

pub fn create_profile_handler() -> Arc<ThreadSafeLobbyHandler> {
    Arc::new(ProfileHandler::new(Arc::new(DwProfileService::new())))
}
