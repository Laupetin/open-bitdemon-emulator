use bitdemon::lobby::ThreadSafeLobbyHandler;
use bitdemon::lobby::matchmaking::MatchmakingHandler;
use std::sync::Arc;

pub fn create_matchmaking_handler() -> Arc<ThreadSafeLobbyHandler> {
    Arc::new(MatchmakingHandler::new())
}
