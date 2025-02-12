mod storage;

use crate::lobby::storage::create_storage_handler;
use bitdemon::lobby::service::anti_cheat::AntiCheatHandler;
use bitdemon::lobby::service::bandwidth::BandwidthHandler;
use bitdemon::lobby::service::league::LeagueHandler;
use bitdemon::lobby::service::title_utilities::TitleUtilitiesHandler;
use bitdemon::lobby::LobbyServer;
use bitdemon::lobby::LobbyServiceId::{Anticheat, BandwidthTest, League, Storage, TitleUtilities};
use std::sync::Arc;

pub fn configure_lobby_server(lobby_server: &LobbyServer) {
    lobby_server.add_service(Anticheat, Arc::new(AntiCheatHandler::new()));
    lobby_server.add_service(BandwidthTest, Arc::new(BandwidthHandler::new()));
    lobby_server.add_service(League, Arc::new(LeagueHandler::new()));
    lobby_server.add_service(Storage, create_storage_handler());
    lobby_server.add_service(TitleUtilities, Arc::new(TitleUtilitiesHandler::new()));
}
