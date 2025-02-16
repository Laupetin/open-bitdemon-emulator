mod service;

use crate::lobby::rich_presence::service::DwRichPresenceService;
use bitdemon::lobby::rich_presence::RichPresenceHandler;
use bitdemon::lobby::ThreadSafeLobbyHandler;
use bitdemon::networking::session_manager::SessionManager;
use std::sync::Arc;

pub fn create_rich_presence_handler(
    session_manager: Arc<SessionManager>,
) -> Arc<ThreadSafeLobbyHandler> {
    Arc::new(RichPresenceHandler::new(DwRichPresenceService::new(
        session_manager,
    )))
}
