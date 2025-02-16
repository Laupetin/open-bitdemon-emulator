mod lobby;
mod log;

use crate::lobby::configure_lobby_server;
use crate::log::{initialize_log, log_session_id};
use bitdemon::auth::auth_server::AuthServer;
use bitdemon::auth::key_store::InMemoryKeyStore;
use bitdemon::lobby::LobbyServer;
use bitdemon::networking::bd_socket::BdSocket;
use bitdemon::networking::session_manager::SessionManager;
use std::sync::Arc;

const AUTH_SERVER_PORT: u16 = 3075;
const LOBBY_SERVER_PORT: u16 = 3074;

fn main() {
    initialize_log();

    let auth_session_manager = Arc::new(SessionManager::new());
    log_session_id(auth_session_manager.as_ref(), "auth");
    let mut auth_socket =
        match BdSocket::new_with_session_manager(AUTH_SERVER_PORT, auth_session_manager) {
            Err(err) => {
                panic!("Failed to open socket for auth server on port {AUTH_SERVER_PORT}: {err}")
            }
            Ok(s) => s,
        };

    let lobby_session_manager = Arc::new(SessionManager::new());
    log_session_id(lobby_session_manager.as_ref(), "lobby");
    let mut lobby_socket = match BdSocket::new_with_session_manager(
        LOBBY_SERVER_PORT,
        lobby_session_manager.clone(),
    ) {
        Err(err) => {
            panic!("Failed to open socket for lobby server on port {LOBBY_SERVER_PORT}: {err}")
        }
        Ok(s) => s,
    };

    let key_store = Arc::new(InMemoryKeyStore::new());

    let auth_server = Arc::new(AuthServer::new(key_store.clone()));
    let lobby_server = Arc::new(LobbyServer::new(key_store.clone()));

    configure_lobby_server(&lobby_server, lobby_session_manager);

    let auth_join = auth_socket.run_async(auth_server);
    let lobby_join = lobby_socket.run_async(lobby_server);

    auth_join.join().unwrap().unwrap();
    lobby_join.join().unwrap().unwrap();
}
