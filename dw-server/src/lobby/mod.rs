use bitdemon::lobby::service::anti_cheat::AntiCheatHandler;
use bitdemon::lobby::service::bandwidth::BandwidthHandler;
use bitdemon::lobby::LobbyServer;
use bitdemon::lobby::LobbyServiceId::{Anticheat, BandwidthTest};
use std::sync::Arc;

pub fn configure_lobby_server(lobby_server: &LobbyServer) {
    lobby_server.add_service(Anticheat, Arc::new(AntiCheatHandler::new()));
    lobby_server.add_service(BandwidthTest, Arc::new(BandwidthHandler::new()));
}
