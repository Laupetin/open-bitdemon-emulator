use crate::networking::bd_session::BdSession;
use std::error::Error;
use std::io::Write;

pub struct BdResponse {
    data: Vec<u8>,
}

impl BdResponse {
    pub fn new(data: Vec<u8>) -> Self {
        BdResponse { data }
    }

    pub fn send(&self, session: &mut BdSession) -> Result<(), Box<dyn Error>> {
        session.write(self.data.as_slice())?;
        Ok(())
    }
}
