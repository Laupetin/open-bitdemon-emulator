use std::io;
use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};

pub type SessionId = u64;

pub struct BdSession {
    pub id: SessionId,
    stream: BufReader<TcpStream>,
    pub session_key: Option<[u8; 24]>,
}

impl io::Read for BdSession {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl io::Write for BdSession {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.get_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.get_mut().flush()
    }
}

impl BdSession {
    pub fn new(stream: TcpStream) -> Self {
        let reader = BufReader::new(stream);

        BdSession {
            id: 0,
            stream: reader,
            session_key: None,
        }
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.stream.get_ref().peer_addr()
    }
}
