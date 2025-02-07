pub mod auth_handler;
pub mod response;
mod result;

use crate::auth_handler::steam::SteamAuthHandler;
use crate::auth_handler::{AuthHandler, AuthMessageType};
use bitdemon::messaging::bd_message::BdMessage;
use bitdemon::networking::bd_session::BdSession;
use bitdemon::networking::bd_socket::BdMessageHandler;
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

pub struct AuthServer {
    auth_handlers: RwLock<HashMap<AuthMessageType, Arc<dyn AuthHandler + Sync + Send>>>,
}

impl AuthServer {
    pub fn new() -> Self {
        let mut handlers: HashMap<AuthMessageType, Arc<dyn AuthHandler + Sync + Send>> =
            HashMap::new();

        handlers.insert(
            AuthMessageType::SteamForMmpRequest,
            Arc::new(SteamAuthHandler::new()),
        );

        AuthServer {
            auth_handlers: RwLock::new(handlers),
        }
    }
}

impl BdMessageHandler for AuthServer {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<(), Box<dyn Error>> {
        let a = message.reader.read_u8()?;

        let handler_type = AuthMessageType::from_u8(a).unwrap();

        let handlers = self.auth_handlers.read().unwrap();
        let handler = handlers.get(&handler_type).unwrap();

        let auth_response = handler.handle_message(session, message)?;
        auth_response.response()?.send(session)?;

        Ok(())
    }
}
