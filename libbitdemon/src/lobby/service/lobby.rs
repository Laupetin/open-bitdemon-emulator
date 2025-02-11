use crate::auth::auth_proof::ClientOpaqueAuthProof;
use crate::domain::title::Title;
use crate::lobby::response::lsg_reply::ConnectionIdResponse;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::StreamMode::BitMode;
use crate::networking::bd_session::BdSession;
use log::info;
use num_traits::FromPrimitive;
use snafu::{OptionExt, Snafu};
use std::error::Error;

pub struct LobbyServiceHandler {}

impl LobbyServiceHandler {
    pub fn new() -> LobbyServiceHandler {
        LobbyServiceHandler {}
    }
}

#[derive(Debug, Snafu)]
enum LobbyServiceError {
    #[snafu(display("The title id is unknown (value={title_id})"))]
    UnknownTitleError { title_id: u32 },
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
        let _title = Title::from_u32(title_id).with_context(|| UnknownTitleSnafu { title_id })?;
        let _iv_seed = message.reader.read_u32()?;

        let mut auth_proof: [u8; 128] = [0; 128];
        message.reader.read_bytes(&mut auth_proof)?;

        let auth_proof = ClientOpaqueAuthProof::deserialize(&auth_proof)?;
        session.session_key = Some(auth_proof.session_key);

        // TODO: Check titleId, expires
        info!(
            "[Session {}] Authenticated with opaque data user_id={} username={}",
            session.id, auth_proof.user_id, auth_proof.username
        );

        Ok(ConnectionIdResponse::new(session.id).to_response()?)
    }
}
