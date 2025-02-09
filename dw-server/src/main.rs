use auth_server::AuthServer;
use bitdemon::networking::bd_socket::BdSocket;
use lobby_server::LobbyServer;
use log::LevelFilter;
use std::sync::Arc;

const AUTH_SERVER_PORT: u16 = 3075;
const LOBBY_SERVER_PORT: u16 = 3074;

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let mut auth_socket = match BdSocket::new(AUTH_SERVER_PORT) {
        Err(err) => {
            panic!("Failed to open socket for auth server on port {AUTH_SERVER_PORT}: {err}")
        }
        Ok(s) => s,
    };

    let mut lobby_socket = match BdSocket::new(LOBBY_SERVER_PORT) {
        Err(err) => {
            panic!("Failed to open socket for lobby server on port {LOBBY_SERVER_PORT}: {err}")
        }
        Ok(s) => s,
    };

    let auth_server = Arc::new(AuthServer::new());
    auth_socket.run(auth_server).unwrap();

    let lobby_server = Arc::new(LobbyServer::new());
    lobby_socket.run(lobby_server).unwrap();
}
