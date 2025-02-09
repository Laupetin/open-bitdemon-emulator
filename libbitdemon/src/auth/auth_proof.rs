use crate::domain::title::Title;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_traits::{FromPrimitive, ToPrimitive};
use snafu::{ensure, Snafu};
use std::error::Error;
use std::io::{Cursor, Read, Write};

/// This represents data that is opaque data that is given to the client that it can use to
/// authenticate to the lobby server.
/// It is encrypted using a key that is only known server side, so the client does not know
/// what is contained within.
/// The data given to the client must be exactly 128 bytes big.
pub struct ClientOpaqueAuthProof {
    pub title: Title,
    pub time_expires: i64,
    pub license_id: u64,
    pub user_id: u64,
    pub session_key: [u8; 24],
    pub username: String,
}

const MAGIC: u32 = 0xBEBEABAB;

#[derive(Debug, Snafu)]
enum AuthProofError {
    #[snafu(display("The magic value for AuthProof is wrong (value={value} expected={MAGIC})"))]
    InvalidMagicError { value: u32 },
    #[snafu(display("The title id is unknown (value={title_id})"))]
    UnknownTitleError { title_id: u32 },
}

impl ClientOpaqueAuthProof {
    pub fn serialize(&self) -> [u8; 128] {
        // TODO: This must be encrypted

        let mut vec = Vec::new();
        let mut cursor = Cursor::new(&mut vec);

        cursor.write_u32::<LittleEndian>(MAGIC).unwrap();
        cursor.write_u32::<LittleEndian>(1).unwrap(); // keyId

        cursor
            .write_u32::<LittleEndian>(self.title.to_u32().unwrap())
            .unwrap();
        cursor.write_i64::<LittleEndian>(self.time_expires).unwrap();
        cursor.write_u64::<LittleEndian>(self.license_id).unwrap();
        cursor.write_u64::<LittleEndian>(self.user_id).unwrap();
        cursor.write(&self.session_key).unwrap();

        let username_bytes = self.username.as_bytes();
        cursor.write(username_bytes).unwrap();
        for _ in username_bytes.len()..64 {
            cursor.write_u8(0).unwrap();
        }

        // Pad
        cursor.write_u32::<LittleEndian>(0).unwrap();

        debug_assert_eq!(vec.len(), 128usize);
        vec.try_into().unwrap()
    }

    pub fn deserialize(buf: &[u8; 128]) -> Result<Self, Box<dyn Error>> {
        let mut cursor = Cursor::new(buf);

        let magic = cursor.read_u32::<LittleEndian>()?;
        ensure!(magic == MAGIC, InvalidMagicSnafu { value: magic });

        let _key_id = cursor.read_u32::<LittleEndian>()?;

        let title_id = cursor.read_u32::<LittleEndian>()?;
        let title =
            Title::from_u32(title_id).ok_or_else(|| UnknownTitleSnafu { title_id }.build())?;
        let time_expires = cursor.read_i64::<LittleEndian>()?;
        let license_id = cursor.read_u64::<LittleEndian>()?;
        let user_id = cursor.read_u64::<LittleEndian>()?;

        let mut session_key: [u8; 24] = [0; 24];
        cursor.read_exact(&mut session_key)?;

        let mut username_buffer: [u8; 64] = [0; 64];
        cursor.read_exact(&mut username_buffer)?;
        let username_end = username_buffer.iter().position(|&v| v == 0).unwrap_or(64);

        let username = String::from_utf8(Vec::from(&username_buffer[0..username_end]))?;

        // Pad
        cursor.read_u32::<LittleEndian>()?;

        Ok(ClientOpaqueAuthProof {
            title,
            time_expires,
            license_id,
            user_id,
            session_key,
            username,
        })
    }
}
