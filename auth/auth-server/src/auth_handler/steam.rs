use crate::auth_handler::AuthHandler;
use bitdemon::networking::bd_message::BdMessage;
use bitdemon::networking::bd_session::BdSession;
use log::info;

pub struct SteamAuthHandler {}

impl AuthHandler for SteamAuthHandler {
    fn handle_message(&self, _session: &mut BdSession, _message: BdMessage) {
        info!("Trying to auth with Steam!");
    }
}

impl SteamAuthHandler {
    pub fn new() -> Self {
        SteamAuthHandler {}
    }
}
