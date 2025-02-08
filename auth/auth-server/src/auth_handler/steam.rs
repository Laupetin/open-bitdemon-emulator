use crate::auth_handler::authentication_request::{
    AuthenticationRequest, SteamAuthenticationRequest,
};
use crate::auth_handler::{AuthHandler, AuthMessageType};
use crate::response::auth_response::AuthResponse;
use crate::result::auth_proof::ClientOpaqueAuthProof;
use crate::result::auth_ticket::{AuthTicket, BdAuthTicketType};
use bitdemon::crypto::{generate_iv_from_seed, generate_iv_seed};
use bitdemon::messaging::bd_message::BdMessage;
use bitdemon::messaging::bd_serialization::{BdDeserialize, BdSerialize};
use bitdemon::messaging::bd_writer::BdWriter;
use bitdemon::messaging::{BdErrorCode, StreamMode};
use bitdemon::networking::bd_session::BdSession;
use cbc::cipher::block_padding::ZeroPadding;
use cbc::cipher::{BlockEncryptMut, KeyIvInit};
use chrono::Utc;
use des::cipher::BlockSizeUser;
use log::{debug, info};
use snafu::{ensure, Snafu};
use std::error::Error;

pub struct SteamAuthHandler {}

const MAX_TICKET_LENGTH: usize = 1024usize;
const TICKET_ISSUE_LENGTH: i64 = 5 * 60 * 1000;

#[derive(Debug, Snafu)]
enum SteamAuthError {
    #[snafu(display("Ticket is too long len={ticket_length} max={MAX_TICKET_LENGTH}"))]
    TicketTooLongError { ticket_length: usize },
}

struct SteamAuthResponse {
    ticket: AuthTicket,
    proof: ClientOpaqueAuthProof,
}

type TdesCbcEnc = cbc::Encryptor<des::TdesEde3>;
type TdesCbcDec = cbc::Decryptor<des::TdesEde3>;

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

        let encrypted = TdesCbcEnc::new(&self.ticket.session_key.into(), &iv.into())
            .encrypt_padded_mut::<ZeroPadding>(&mut ticket_buf, ticket_buf_len)
            .unwrap();
        writer.write_bytes(encrypted)?;

        let proof_data = self.proof.serialize();
        writer.write_bytes(&proof_data)?;

        Ok(())
    }
}

impl SteamAuthHandler {
    pub fn new() -> Self {
        SteamAuthHandler {}
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
            "Trying to auth with Steam iv_seed={:x} title={:?} session_key={:?} username={}",
            authentication_request.iv_seed,
            authentication_request.title,
            request_data.session_key,
            &request_data.username
        );

        let now = Utc::now();
        let issued = (now.timestamp() % (u32::MAX as i64)) as u32;
        let expires = ((now.timestamp() + TICKET_ISSUE_LENGTH) % (u32::MAX as i64)) as u32;

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

        let proof = ClientOpaqueAuthProof {};

        Ok(Box::new(SteamAuthResponse { ticket, proof }))
    }
}
