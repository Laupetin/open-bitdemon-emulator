use crate::auth::auth_handler::steam::SteamAuthHandler;
use crate::auth::auth_handler::{AuthHandler, AuthMessageType};
use crate::auth::key_store::ThreadSafeBackendPrivateKeyStorage;
use crate::auth::response::{AuthResponse, AuthResponseWithOnlyCode};
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_response::ResponseCreator;
use crate::messaging::BdErrorCode::AuthIllegalOperation;
use crate::networking::bd_session::BdSession;
use crate::networking::bd_socket::BdMessageHandler;
use log::warn;
use num_traits::FromPrimitive;
use snafu::Snafu;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

pub struct AuthServer {
    auth_handlers: RwLock<HashMap<AuthMessageType, Arc<dyn AuthHandler + Sync + Send>>>,
}

impl AuthServer {
    pub fn new(key_store: Arc<ThreadSafeBackendPrivateKeyStorage>) -> Self {
        let mut handlers: HashMap<AuthMessageType, Arc<dyn AuthHandler + Sync + Send>> =
            HashMap::new();

        handlers.insert(
            AuthMessageType::SteamForMmpRequest,
            Arc::new(SteamAuthHandler::new(key_store)),
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
                auth_response.to_response()?.send(session)?;

                Ok(())
            }
            None => {
                warn!(
                    "[Session {}] Tried to request unavailable auth handler {handler_type:?}",
                    session.id
                );
                let only: Box<dyn AuthResponse> = Box::from(AuthResponseWithOnlyCode::new(
                    handler_type.reply_code(),
                    AuthIllegalOperation,
                ));

                only.to_response()?.send(session)?;

                Ok(())
            }
        }
    }
}
