use crate::messaging::bd_reader::BdReader;
use crate::networking::bd_session::BdSession;
use std::error::Error;

pub struct BdMessage {
    pub reader: BdReader,
}

impl BdMessage {
    pub fn new(_session: &BdSession, buf: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let encrypted = buf.get(0).unwrap();
        if *encrypted > 0 {
            todo!("Encryption not implemented")
        }

        Ok(BdMessage {
            reader: BdReader::new(Vec::from(&buf[1..buf.len()])),
        })
    }
}
