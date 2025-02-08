/// This represents data that is opaque data that is given to the client that it can use to
/// authenticate to the lobby server.
/// It is encrypted using a key that is only known server side, so the client does not know
/// what is contained within.
/// The data given to the client must be exactly 128 bytes big.
pub struct ClientOpaqueAuthProof {}

impl ClientOpaqueAuthProof {
    pub fn serialize(&self) -> [u8; 128] {
        [0u8; 128]
    }
}
