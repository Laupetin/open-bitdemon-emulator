mod config;
mod lobby;
mod log;

use crate::config::DwServerConfig;
use crate::lobby::configure_lobby_server;
use crate::log::{initialize_log, log_session_id};
use ::log::{error, info};
use bitdemon::auth::auth_server::AuthServer;
use bitdemon::auth::key_store::InMemoryKeyStore;
use bitdemon::lobby::LobbyServer;
use bitdemon::networking::bd_socket::BdSocket;
use bitdemon::networking::session_manager::SessionManager;
use std::process::exit;
use std::sync::Arc;
use tokio::fs::read_to_string;
use tokio::net::TcpListener;

const AUTH_SERVER_PORT: u16 = 3075;
const LOBBY_SERVER_PORT: u16 = 3074;

#[tokio::main]
async fn main() {
    initialize_log();

    let config = read_config().await;

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

    let lobby_router = configure_lobby_server(&lobby_server, lobby_session_manager, &config);

    let auth_join = auth_socket.run_async(auth_server);
    let lobby_join = lobby_socket.run_async(lobby_server);

    let content_port = config.content_port();
    info!("Running content http server on port {content_port}");
    let listener = TcpListener::bind(format!("0.0.0.0:{content_port}"))
        .await
        .unwrap();
    let http_promise = axum::serve(listener, lobby_router);

    http_promise.await.unwrap();
    auth_join.join().unwrap().unwrap();
    lobby_join.join().unwrap().unwrap();
}

async fn read_config() -> DwServerConfig {
    read_config_from_file().await.unwrap_or_else(|| {
        info!("Applying default configuration");
        DwServerConfig::default()
    })
}

async fn read_config_from_file() -> Option<DwServerConfig> {
    let json_str = read_to_string("./config.json")
        .await
        .map_err(|_| {
            info!("Could not read config.json, applying default configuration");
        })
        .ok()?;

    let config = serde_json::from_str(json_str.as_str())
        .map_err(|e| {
            error!("Failed to parse config: {e}");
            exit(1);
        })
        .unwrap();

    Some(config)
}
