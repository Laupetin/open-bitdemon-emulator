use std::io::{BufReader, Cursor};

pub struct BdMessage {
    pub reader: Cursor<Vec<u8>>,
}

impl BdMessage {
    pub fn new(buf: Vec<u8>) -> Self {
        let encrypted = buf.get(0).unwrap();
        if *encrypted > 0 {
            todo!("Encryption not implemented")
        }

        let mut cursor = Cursor::new(buf);
        cursor.set_position(1);

        BdMessage { reader: cursor }
    }
}
