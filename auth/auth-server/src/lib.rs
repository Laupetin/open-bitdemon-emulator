pub mod auth_handler;
pub mod response;
mod result;

use crate::auth_handler::steam::SteamAuthHandler;
use crate::auth_handler::{AuthHandler, AuthMessageType};
use crate::response::{AuthResponse, AuthResponseWithOnlyCode};
use bitdemon::messaging::bd_message::BdMessage;
use bitdemon::messaging::BdErrorCode::AuthIllegalOperation;
use bitdemon::networking::bd_session::BdSession;
use bitdemon::networking::bd_socket::BdMessageHandler;
use num_traits::FromPrimitive;
use snafu::{ensure, Snafu};
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

#[derive(Debug, Snafu)]
enum AuthServerError {
    #[snafu(display("The client specified an illegal message type: {message_type_input}"))]
    IllegalMessageTypeError { message_type_input: u8 },
}

impl BdMessageHandler for AuthServer {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<(), Box<dyn Error>> {
        let message_type_input = message.reader.read_u8()?;

        let handler_type = AuthMessageType::from_u8(message_type_input)
            .ok_or_else(|| IllegalMessageTypeSnafu { message_type_input }.build())?;

        let handlers = self.auth_handlers.read().unwrap();
        let maybe_handler = handlers.get(&handler_type);

        match maybe_handler {
            Some(handler) => {
                let auth_response = handler.handle_message(session, message)?;
                auth_response.response()?.send(session)?;

                Ok(())
            }
            None => {
                let only: Box<dyn AuthResponse> = Box::from(AuthResponseWithOnlyCode::new(
                    handler_type.reply_code(),
                    AuthIllegalOperation,
                ));

                only.response()?.send(session)?;

                Ok(())
            }
        }
    }
}
