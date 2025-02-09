use auth_server::AuthServer;
use bitdemon::networking::bd_socket::BdSocket;
use log::LevelFilter;
use std::sync::Arc;

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let mut socket = match BdSocket::new(3075) {
        Err(err) => panic!("Failed to open socket: {}", err),
        Ok(s) => s,
    };

    let auth_server = Arc::new(AuthServer::new());
    socket.run_sync(auth_server).unwrap();
}
