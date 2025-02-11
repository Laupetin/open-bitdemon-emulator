use bitdemon::auth::AuthServer;
use bitdemon::lobby::LobbyServer;
use bitdemon::networking::bd_socket::BdSocket;
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
    let auth_join = auth_socket.run_async(auth_server);

    let lobby_server = Arc::new(LobbyServer::new());
    let lobby_join = lobby_socket.run_async(lobby_server);

    auth_join.join().unwrap().unwrap();
    lobby_join.join().unwrap().unwrap();
}
