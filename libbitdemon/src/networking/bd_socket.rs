use crate::messaging::bd_message::BdMessage;
use crate::networking::bd_session::BdSession;
use crate::networking::session_manager::SessionManager;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use log::{error, info};
use snafu::{ensure, Snafu};
use std::error::Error;
use std::io::{ErrorKind, Read};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{io, thread};

const MAX_MESSAGE_SIZE: u32 = 0x4000000;

#[derive(Debug, Snafu)]
enum BdSocketError {
    #[snafu(display("Message was too large (size={msg_size}, max={MAX_MESSAGE_SIZE})"))]
    MessageTooLargeError { msg_size: u32 },
    #[snafu(display("The client sent an incomplete message header"))]
    IncompleteMessageHeaderError {},
}

pub trait BdMessageHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        message: BdMessage,
    ) -> Result<(), Box<dyn Error>>;
}

pub struct BdSocket {
    listener: Option<TcpListener>,
}

impl BdSocket {
    /// Creates a new BdSocket instance and binds it to the specified port.
    pub fn new(port: u16) -> Result<BdSocket, io::Error> {
        let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;

        info!("Opened bitdemon socket on port {port}");

        Ok(BdSocket {
            listener: Some(listener),
        })
    }

    fn listen(
        listener: &TcpListener,
        message_handler: Arc<dyn BdMessageHandler + Send + Sync>,
    ) -> Result<(), io::Error> {
        let session_manager = Arc::new(SessionManager::new());
        for stream in listener.incoming() {
            let stream = stream?;

            let session_manager = Arc::clone(&session_manager);
            let message_handler = Arc::clone(&message_handler);
            thread::spawn(move || {
                let mut session = BdSession::new(stream);
                session_manager.register_session(&mut session);
                BdSocket::handle_connection(&mut session, message_handler.as_ref());
                session_manager.unregister_session(&session);
            });
        }

        Ok(())
    }

    pub fn run_sync(
        &mut self,
        message_handler: Arc<dyn BdMessageHandler + Send + Sync>,
    ) -> Result<(), io::Error> {
        Self::listen(self.listener.as_ref().unwrap(), message_handler)
    }

    pub fn run_async(
        &mut self,
        message_handler: Arc<dyn BdMessageHandler + Send + Sync>,
    ) -> JoinHandle<Result<(), io::Error>> {
        let message_handler = Arc::clone(&message_handler);
        let listener = self.listener.take();
        thread::spawn(move || -> Result<(), io::Error> {
            Self::listen(listener.as_ref().unwrap(), message_handler)
        })
    }

    fn handle_connection(session: &mut BdSession, message_handler: &dyn BdMessageHandler) {
        let connection_loop = |session: &mut BdSession| -> Result<(), Box<dyn Error>> {
            loop {
                let mut b: [u8; 4] = [0; 4];
                let len = session.read(&mut b)?;
                if len == 0 {
                    return Ok(());
                }

                ensure!(len == 4, IncompleteMessageHeaderSnafu {});
                let header = u32::from_le_bytes(b);

                match header {
                    0 => {
                        info!("[Session {}] Ping", session.id);
                        session.write_u32::<LittleEndian>(0)?;
                    }
                    200 => {
                        let available_buffer_size = session.read_u32::<LittleEndian>()?;
                        info!(
                            "[Session {}] Buffer available: {available_buffer_size}",
                            session.id
                        );
                    }
                    _ => {
                        ensure!(
                            header <= MAX_MESSAGE_SIZE,
                            MessageTooLargeSnafu { msg_size: header }
                        );

                        info!("[Session {}] Message with size {header}", session.id);
                        let mut msg = vec![0; header as usize];
                        session.read(msg.as_mut_slice())?;
                        let message = BdMessage::new(&session, msg)?;
                        message_handler.handle_message(session, message)?;
                    }
                }
            }
        };

        let connection_result = connection_loop(session);
        match connection_result {
            Err(e) => {
                if let Some(e0) = e.downcast_ref::<io::Error>() {
                    match e0.kind() {
                        ErrorKind::Interrupted | ErrorKind::ConnectionReset => {}
                        _ => error!(
                            "[Session {}] Connection terminated: {}: {e}",
                            session.id,
                            e0.kind()
                        ),
                    }
                } else {
                    error!(
                        "[Session {}] Session terminated with error: {e}",
                        session.id
                    )
                }
            }
            Ok(_) => (),
        }
    }
}
