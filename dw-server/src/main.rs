mod lobby;

use crate::lobby::configure_lobby_server;
use bitdemon::auth::auth_server::AuthServer;
use bitdemon::auth::key_store::InMemoryKeyStore;
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

    let key_store = Arc::new(InMemoryKeyStore::new());

    let auth_server = Arc::new(AuthServer::new(key_store.clone()));
    let lobby_server = Arc::new(LobbyServer::new(key_store.clone()));

    configure_lobby_server(&lobby_server);

    let auth_join = auth_socket.run_async(auth_server);
    let lobby_join = lobby_socket.run_async(lobby_server);

    auth_join.join().unwrap().unwrap();
    lobby_join.join().unwrap().unwrap();
}
