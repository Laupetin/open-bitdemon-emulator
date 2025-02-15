use bitdemon::lobby::group::GroupHandler;
use bitdemon::lobby::ThreadSafeLobbyHandler;
use bitdemon::networking::session_manager::SessionManager;
use std::sync::Arc;

mod service;

pub fn create_group_handler(session_manager: Arc<SessionManager>) -> Arc<ThreadSafeLobbyHandler> {
    Arc::new(GroupHandler::new(service::DwGroupService::new(
        session_manager,
    )))
}
