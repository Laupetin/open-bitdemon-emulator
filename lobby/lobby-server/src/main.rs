use bitdemon::networking::bd_socket::BdSocket;
use lobby_server::LobbyServer;
use log::LevelFilter;
use std::sync::Arc;

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let mut socket = match BdSocket::new(3074) {
        Err(err) => panic!("Failed to open socket: {}", err),
        Ok(s) => s,
    };

    socket.run_sync(Arc::new(LobbyServer::new())).unwrap();
}
