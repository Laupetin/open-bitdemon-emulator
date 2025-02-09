use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Cursor, Write};

/// This represents data that is opaque data that is given to the client that it can use to
/// authenticate to the lobby server.
/// It is encrypted using a key that is only known server side, so the client does not know
/// what is contained within.
/// The data given to the client must be exactly 128 bytes big.
pub struct ClientOpaqueAuthProof {}

const MAGIC: u32 = 0xBEBEABAB;

impl ClientOpaqueAuthProof {
    pub fn serialize(&self) -> [u8; 128] {
        let mut vec = Vec::new();
        let mut cursor = Cursor::new(&mut vec);

        cursor.write_u32::<LittleEndian>(MAGIC).unwrap();
        cursor.write_u32::<LittleEndian>(1).unwrap();
        cursor.write(&[0u8; 120]).unwrap();

        vec.try_into().unwrap()
    }
}
