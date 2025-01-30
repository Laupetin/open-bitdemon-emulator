use crate::messaging::bd_message::BdMessage;
use crate::networking::bd_session::BdSession;

pub trait BdServer {
    fn handle_message(&self, session: BdSession, message: BdMessage);
}
