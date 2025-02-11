use crate::auth::auth_handler::authentication_request::{
    AuthenticationRequest, SteamAuthenticationRequest,
};
use crate::auth::auth_handler::{AuthHandler, AuthMessageType};
use crate::auth::auth_proof::ClientOpaqueAuthProof;
use crate::auth::key_store::ThreadSafeBackendPrivateKeyStorage;
use crate::auth::response::AuthResponse;
use crate::auth::result::auth_ticket::{AuthTicket, BdAuthTicketType};
use crate::crypto::{encrypt_buffer_in_place, generate_iv_from_seed, generate_iv_seed};
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_serialization::{BdDeserialize, BdSerialize};
use crate::messaging::bd_writer::BdWriter;
use crate::messaging::{BdErrorCode, StreamMode};
use crate::networking::bd_session::BdSession;
use chrono::Utc;
use des::cipher::BlockSizeUser;
use log::info;
use std::error::Error;
use std::sync::Arc;

pub struct SteamAuthHandler {
    key_store: Arc<ThreadSafeBackendPrivateKeyStorage>,
}

const TICKET_ISSUE_LENGTH: i64 = 5 * 60 * 1000;

struct SteamAuthResponse {
    ticket: AuthTicket,
    serialized_proof_data: [u8; 128],
}

impl AuthResponse for SteamAuthResponse {
    fn message_type(&self) -> AuthMessageType {
        AuthMessageType::SteamForMmpReply
    }

    fn error_code(&self) -> BdErrorCode {
        BdErrorCode::AuthNoError
    }

    fn write_auth_data(&self, writer: &mut BdWriter) -> Result<(), Box<dyn Error>> {
        let seed = generate_iv_seed();
        writer.write_u32(seed)?;

        let mut ticket_buf = Vec::new();
        {
            let mut ticket_writer = BdWriter::new(&mut ticket_buf);
            self.ticket.serialize(&mut ticket_writer)?;
        }

        let iv = generate_iv_from_seed(seed);
        let ticket_buf_len = ticket_buf.len();
        ticket_buf.resize(
            ticket_buf_len.next_multiple_of(des::TdesEde3::block_size()),
            0,
        );

        encrypt_buffer_in_place(&mut ticket_buf, &self.ticket.session_key, &iv);
        writer.write_bytes(ticket_buf.as_slice())?;

        writer.write_bytes(&self.serialized_proof_data)?;

        Ok(())
    }
}

impl SteamAuthHandler {
    pub fn new(key_store: Arc<ThreadSafeBackendPrivateKeyStorage>) -> Self {
        SteamAuthHandler { key_store }
    }
}

impl AuthHandler for SteamAuthHandler {
    fn handle_message(
        &self,
        _session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<Box<dyn AuthResponse>, Box<dyn Error>> {
        message.reader.set_mode(StreamMode::BitMode);
        message.reader.read_type_checked_bit()?;

        let authentication_request = AuthenticationRequest::deserialize(&mut message.reader)?;
        let request_data = match authentication_request.request_data {
            SteamAuthenticationRequest::Custom { request_data: t } => t,
        };

        info!(
            "Trying to auth with Steam iv_seed={:x} title={:?} username={}",
            authentication_request.iv_seed, authentication_request.title, &request_data.username
        );

        let now = Utc::now();
        let issued = (now.timestamp() % (u32::MAX as i64)) as u32;
        let expires_i64 = now.timestamp() + TICKET_ISSUE_LENGTH;
        let expires = ((expires_i64) % (u32::MAX as i64)) as u32;

        let ticket = AuthTicket {
            ticket_type: BdAuthTicketType::UserToServiceTicket,
            title: authentication_request.title,
            time_issued: issued,
            time_expires: expires,
            license_id: 1234u64,
            user_id: request_data.steam_id,
            username: request_data.username,
            session_key: request_data.session_key,
        };

        let proof = ClientOpaqueAuthProof {
            title: ticket.title,
            time_expires: expires_i64,
            license_id: ticket.license_id,
            user_id: ticket.user_id,
            session_key: ticket.session_key,
            username: String::from(&ticket.username),
        };
        let serialized_proof_data = proof.serialize(self.key_store.as_ref());

        Ok(Box::new(SteamAuthResponse {
            ticket,
            serialized_proof_data,
        }))
    }
}
