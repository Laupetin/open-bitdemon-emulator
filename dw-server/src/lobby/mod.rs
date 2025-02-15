mod counter;
mod storage;

use crate::lobby::counter::create_counter_handler;
use crate::lobby::storage::create_storage_handler;
use bitdemon::lobby::anti_cheat::AntiCheatHandler;
use bitdemon::lobby::bandwidth::BandwidthHandler;
use bitdemon::lobby::dml::DmlHandler;
use bitdemon::lobby::league::LeagueHandler;
use bitdemon::lobby::title_utilities::TitleUtilitiesHandler;
use bitdemon::lobby::LobbyServer;
use bitdemon::lobby::LobbyServiceId::{
    Anticheat, BandwidthTest, Counter, Dml, League, Storage, TitleUtilities,
};
use std::sync::Arc;

pub fn configure_lobby_server(lobby_server: &LobbyServer) {
    lobby_server.add_service(Anticheat, Arc::new(AntiCheatHandler::new()));
    lobby_server.add_service(BandwidthTest, Arc::new(BandwidthHandler::new()));
    lobby_server.add_service(Counter, create_counter_handler());
    lobby_server.add_service(Dml, Arc::new(DmlHandler::new()));
    lobby_server.add_service(League, Arc::new(LeagueHandler::new()));
    lobby_server.add_service(Storage, create_storage_handler());
    lobby_server.add_service(TitleUtilities, Arc::new(TitleUtilitiesHandler::new()));
}
