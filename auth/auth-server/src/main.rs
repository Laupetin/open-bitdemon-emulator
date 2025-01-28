mod auth_handler;

use crate::auth_handler::steam::SteamAuthHandler;
use crate::auth_handler::{AuthHandler, AuthHandlerType};
use bitdemon::networking::bd_message::BdMessage;
use bitdemon::networking::bd_session::BdSession;
use bitdemon::networking::bd_socket::{BdMessageHandler, BdSocket};
use byteorder::{LittleEndian, ReadBytesExt};
use log::{info, LevelFilter};
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct AuthServer {
    auth_handlers: RwLock<HashMap<AuthHandlerType, Arc<dyn AuthHandler + Sync + Send>>>,
}

impl AuthServer {
    pub fn new() -> Self {
        let mut handlers: HashMap<AuthHandlerType, Arc<dyn AuthHandler + Sync + Send>> =
            HashMap::new();

        handlers.insert(
            AuthHandlerType::SteamForMmpRequest,
            Arc::new(SteamAuthHandler::new()),
        );

        AuthServer {
            auth_handlers: RwLock::new(handlers),
        }
    }
}

impl BdMessageHandler for AuthServer {
    fn handle_message(&self, session: &mut BdSession, mut message: BdMessage) {
        let a = message.reader.read_u8().unwrap();

        let handler_type = AuthHandlerType::from_u8(a).unwrap();

        let handlers = self.auth_handlers.read().unwrap();
        let handler = handlers.get(&handler_type).unwrap();

        handler.handle_message(session, message);
    }
}

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let mut socket = match BdSocket::new(3075) {
        Err(err) => panic!("Failed to open socket: {}", err),
        Ok(s) => s,
    };

    let auth_server = Arc::new(AuthServer::new());
    socket.run(auth_server).unwrap();
}
