use crate::auth::auth_proof::ClientOpaqueAuthProof;
use crate::auth::authentication::SessionAuthentication;
use crate::auth::key_store::ThreadSafeBackendPrivateKeyStorage;
use crate::domain::title::Title;
use crate::lobby::response::lsg_reply::ConnectionIdResponse;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::StreamMode::BitMode;
use crate::networking::bd_session::BdSession;
use log::info;
use num_traits::FromPrimitive;
use snafu::{ensure, OptionExt, Snafu};
use std::error::Error;
use std::sync::Arc;

pub struct LobbyServiceHandler {
    key_store: Arc<ThreadSafeBackendPrivateKeyStorage>,
}

impl LobbyServiceHandler {
    pub fn new(key_store: Arc<ThreadSafeBackendPrivateKeyStorage>) -> LobbyServiceHandler {
        LobbyServiceHandler { key_store }
    }
}

#[derive(Debug, Snafu)]
enum LobbyServiceError {
    #[snafu(display("The title id is unknown (value={title_id})"))]
    UnknownTitleError { title_id: u32 },
    #[snafu(display("The specified title id does not match (specified_title={specified_title:?} authenticated_title={authenticated_title:?})"))]
    InvalidTitleError {
        specified_title: Title,
        authenticated_title: Title,
    },
    #[snafu(display("The authentication expired (expires={expires} now={now})"))]
    AuthenticationExpiredError { expires: i64, now: i64 },
}

impl LobbyHandler for LobbyServiceHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        message.reader.set_mode(BitMode);
        message.reader.read_type_checked_bit()?;

        let title_id = message.reader.read_u32()?;
        let title = Title::from_u32(title_id).with_context(|| UnknownTitleSnafu { title_id })?;
        let _iv_seed = message.reader.read_u32()?;

        let mut auth_proof: [u8; 128] = [0; 128];
        message.reader.read_bytes(&mut auth_proof)?;

        let auth_proof =
            ClientOpaqueAuthProof::deserialize(&mut auth_proof, self.key_store.as_ref())?;

        let now = chrono::Utc::now().timestamp();
        ensure!(
            auth_proof.time_expires >= now,
            AuthenticationExpiredSnafu {
                expires: auth_proof.time_expires,
                now
            }
        );

        ensure!(
            auth_proof.title == title,
            InvalidTitleSnafu {
                specified_title: title,
                authenticated_title: auth_proof.title
            }
        );

        info!(
            "[Session {}] Authenticated with opaque data user_id={} username={}",
            session.id, auth_proof.user_id, auth_proof.username
        );

        session.authentication = Some(SessionAuthentication {
            user_id: auth_proof.user_id,
            username: auth_proof.username,
            session_key: auth_proof.session_key,
            title: auth_proof.title,
        });

        Ok(ConnectionIdResponse::new(session.id).to_response()?)
    }
}
