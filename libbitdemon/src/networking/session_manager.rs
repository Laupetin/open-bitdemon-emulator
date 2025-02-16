use crate::networking::bd_session::{BdSession, SessionId};
use log::info;
use std::sync::Mutex;

type OnSessionCallback = dyn FnMut(&BdSession) + Sync + Send;

pub struct SessionManager {
    session_id_counter: Mutex<SessionId>,
    register_cb: Mutex<Vec<Box<OnSessionCallback>>>,
    unregister_cb: Mutex<Vec<Box<OnSessionCallback>>>,
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager {
            session_id_counter: Mutex::new(0),
            register_cb: Mutex::new(vec![]),
            unregister_cb: Mutex::new(vec![]),
        }
    }

    pub fn register_session(&self, session: &mut BdSession) {
        let mut session_counter = self.session_id_counter.lock().unwrap();
        session.id = *session_counter;
        *session_counter += 1;

        let peer_addr = session.peer_addr().unwrap();
        info!(
            "New session {} from {}:{}",
            session.id,
            peer_addr.ip(),
            peer_addr.port()
        );

        self.register_cb
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|cb| cb(session));
    }

    pub fn unregister_session(&self, session: &BdSession) {
        info!("Session ended");

        self.unregister_cb
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|cb| cb(session));
    }

    pub fn on_session_registered<F>(&self, cb: F)
    where
        F: FnMut(&BdSession) + Sync + Send + 'static,
    {
        self.register_cb.lock().unwrap().push(Box::from(cb));
    }

    pub fn on_session_unregistered<F>(&self, cb: F)
    where
        F: FnMut(&BdSession) + Sync + Send + 'static,
    {
        self.unregister_cb.lock().unwrap().push(Box::from(cb));
    }
}
