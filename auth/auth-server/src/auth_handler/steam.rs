use crate::auth_handler::AuthHandler;
use bitdemon::messaging::bd_message::BdMessage;
use bitdemon::messaging::StreamMode;
use bitdemon::networking::bd_session::BdSession;
use log::info;

pub struct SteamAuthHandler {}

impl AuthHandler for SteamAuthHandler {
    fn handle_message(&self, _session: &mut BdSession, mut message: BdMessage) {
        message.reader.set_mode(StreamMode::BitMode);
        message.reader.read_type_checked_bit().unwrap();

        let iv_seed = message.reader.read_u32().unwrap();
        let title_id = message.reader.read_u32().unwrap();

        let ticket_length = message.reader.read_u32().unwrap();

        info!("Trying to auth with Steam iv_seed={iv_seed} title_id={title_id} ticket_length={ticket_length}");
    }
}

impl SteamAuthHandler {
    pub fn new() -> Self {
        SteamAuthHandler {}
    }
}
