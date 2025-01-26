use crate::networking::bd_session::BdSession;
use log::info;
use std::sync::Mutex;

pub struct SessionManager {
    session_id_counter: Mutex<u64>,
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager {
            session_id_counter: Mutex::new(0u64),
        }
    }

    pub fn register_session(&self, session: &mut BdSession) {
        let mut session_counter = self.session_id_counter.lock().unwrap();
        session.id = *session_counter;
        *session_counter += 1;

        let peer_addr = session.peer_addr().unwrap();
        info!(
            "[Session {}] New session from {}:{}",
            session.id,
            peer_addr.ip(),
            peer_addr.port()
        )
    }

    pub fn unregister_session(&self, session: &BdSession) {
        info!("[Session {}] Session ended", session.id)
    }
}
