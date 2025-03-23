use crate::lobby::matchmaking::service::DwMatchmakingService;
use crate::lobby::ConfiguredEnvironment;
use bitdemon::lobby::matchmaking::MatchmakingHandler;
use bitdemon::lobby::LobbyServiceId;
use std::sync::Arc;

mod service;

pub fn create_matchmaking_handler() -> ConfiguredEnvironment {
    ConfiguredEnvironment::new(
        LobbyServiceId::Matchmaking,
        Arc::new(MatchmakingHandler::new(Arc::new(
            DwMatchmakingService::new(),
        ))),
    )
}
