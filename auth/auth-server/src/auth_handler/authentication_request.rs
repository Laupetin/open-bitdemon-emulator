use bitdemon::domain::title::Title;
use bitdemon::messaging::bd_reader::BdReader;
use bitdemon::messaging::bd_serialization::BdDeserialize;
use bitdemon::messaging::StreamMode;
use num_traits::FromPrimitive;
use snafu::{ensure, OptionExt, Snafu};
use std::error::Error;

pub struct AuthenticationRequest {
    pub iv_seed: u32,
    pub title: Title,
    pub request_data: SteamAuthenticationRequest,
}

pub enum AuthenticationRequestData {
    Steam { request: SteamAuthenticationRequest },
}

#[derive(Debug, Snafu)]
enum AuthenticationRequestDeserializationError {
    #[snafu(display("The title id is unknown (value={title_id})"))]
    UnknownTitleError { title_id: u32 },
    #[snafu(display("The request data is too long (len={data_len} max={MAX_DATA_LEN})"))]
    RequestDataTooLongError { data_len: usize },
}

const MAX_DATA_LEN: usize = 128usize;

impl BdDeserialize for AuthenticationRequest {
    fn deserialize(reader: &mut BdReader) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let iv_seed = reader.read_u32()?;
        let title_id = reader.read_u32()?;
        let title = Title::from_u32(title_id).with_context(|| UnknownTitleSnafu { title_id })?;

        let data_len = reader.read_u32()? as usize;
        ensure!(
            data_len <= MAX_DATA_LEN,
            RequestDataTooLongSnafu { data_len }
        );

        let mut data_buf = Vec::new();
        data_buf.resize(data_len, 0u8);

        reader.read_bytes(data_buf.as_mut_slice())?;

        let mut ticket_reader = BdReader::new(data_buf);

        let request_data = SteamAuthenticationRequest::Custom {
            request_data: CustomSteamAuthenticationRequest::deserialize(&mut ticket_reader)?,
        };

        Ok(AuthenticationRequest {
            iv_seed,
            title,
            request_data,
        })
    }
}

pub enum SteamAuthenticationRequest {
    Custom {
        request_data: CustomSteamAuthenticationRequest,
    },
}

pub struct CustomSteamAuthenticationRequest {
    pub steam_id: u64,
    pub session_key: [u8; 24],
    pub username: String,
}

#[derive(Debug, Snafu)]
enum TicketDeserializationError {
    #[snafu(display("The ticket signature did not match (actual={actual} expected={expected})"))]
    SignatureMismatchError { actual: u32, expected: u32 },
    #[snafu(display(
        "The secret data had an unexpected secret length (actual={actual} expected={expected})"
    ))]
    UnexpectedSecretLengthError { actual: usize, expected: usize },
    #[snafu(display("The username has an invalid length (actual={actual} expected={expected})"))]
    UsernameTooLongError { actual: usize, expected: usize },
}

// We cannot decrypt Steam tickets. We can only parse ones that we issued in a custom format.
const CUSTOM_TICKET_SIGNATURE: u32 = 0xDEADBABE;
const EXPECTED_SECRET_DATA_SIZE: usize = 24usize + 64usize;
impl BdDeserialize for CustomSteamAuthenticationRequest {
    fn deserialize(reader: &mut BdReader) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        reader.set_mode(StreamMode::ByteMode);
        reader.set_type_checked(false);

        let signature = reader.read_u32()?;

        ensure!(
            signature == CUSTOM_TICKET_SIGNATURE,
            SignatureMismatchSnafu {
                actual: signature,
                expected: CUSTOM_TICKET_SIGNATURE
            }
        );

        let steam_id = reader.read_u64()?;

        let secret_data_size = reader.read_u32()? as usize;
        ensure!(
            secret_data_size == EXPECTED_SECRET_DATA_SIZE,
            UnexpectedSecretLengthSnafu {
                actual: secret_data_size,
                expected: EXPECTED_SECRET_DATA_SIZE
            }
        );

        let mut session_key: [u8; 24] = [0; 24];
        reader.read_bytes(&mut session_key)?;

        let username = reader.read_str()?;
        ensure!(
            username.len() < 64usize,
            UsernameTooLongSnafu {
                actual: username.len(),
                expected: 64usize
            }
        );

        Ok(CustomSteamAuthenticationRequest {
            steam_id,
            session_key,
            username,
        })
    }
}
