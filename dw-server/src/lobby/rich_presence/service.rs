use bitdemon::lobby::rich_presence::{RichPresenceService, RichPresenceServiceError};
use bitdemon::networking::bd_session::BdSession;
use bitdemon::networking::session_manager::SessionManager;
use log::{info, warn};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct DwRichPresenceService {
    rich_presences: RwLock<HashMap<u64, Vec<u8>>>,
}

const MAX_RICH_PRESENCE_SIZE: usize = 1_024; // 1KiB
const MAX_USER_RICH_PRESENCE_COUNT: usize = 64;

impl RichPresenceService for DwRichPresenceService {
    fn set_info(
        &self,
        session: &BdSession,
        user_id: u64,
        rich_presence_data: Vec<u8>,
    ) -> Result<(), RichPresenceServiceError> {
        info!(
            "Setting rich presence user={user_id} len={}",
            rich_presence_data.len()
        );

        if session.authentication().unwrap().user_id != user_id {
            warn!("Tried to set rich presence for other user");
            return Err(RichPresenceServiceError::PermissionDeniedError);
        }

        if rich_presence_data.len() > MAX_RICH_PRESENCE_SIZE {
            warn!("Tried to set upload rich presence that is too large");
            return Err(RichPresenceServiceError::RichPresenceDataTooLargeError);
        }

        let mut rich_presences = self.rich_presences.write().unwrap();
        rich_presences.insert(user_id, rich_presence_data);

        Ok(())
    }

    fn get_info(
        &self,
        _session: &BdSession,
        users: &[u64],
    ) -> Result<Vec<Option<Vec<u8>>>, RichPresenceServiceError> {
        info!("Retrieving rich presence data for {} users", users.len());

        if users.len() > MAX_USER_RICH_PRESENCE_COUNT {
            warn!("Too many users requested at once");
            return Err(RichPresenceServiceError::TooManyUsersError);
        }

        let mut result = Vec::new();
        result.reserve(users.len());

        let rich_presences = self.rich_presences.read().unwrap();
        for user in users {
            result.push(rich_presences.get(user).cloned());
        }

        Ok(result)
    }
}

impl DwRichPresenceService {
    pub fn new(session_manager: Arc<SessionManager>) -> Arc<DwRichPresenceService> {
        let service = Arc::new(DwRichPresenceService {
            rich_presences: RwLock::new(HashMap::new()),
        });

        Self::register_session_manager_callbacks(service.clone(), session_manager);

        service
    }

    fn register_session_manager_callbacks(
        service: Arc<Self>,
        session_manager: Arc<SessionManager>,
    ) {
        session_manager.on_session_unregistered(move |session| {
            if let Some(authentication) = session.authentication() {
                service.remove_rich_presence_for_disconnect(authentication.user_id);
            }
        });
    }

    fn remove_rich_presence_for_disconnect(&self, user_id: u64) {
        let mut rich_presences = self.rich_presences.write().unwrap();
        if let Some(_) = rich_presences.remove(&user_id) {
            info!(
                "Removed rich presence for user {} due to disconnect",
                user_id
            );
        }
    }
}
