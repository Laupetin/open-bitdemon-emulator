use bitdemon::lobby::matchmaking::MatchmakingService;

pub struct DwMatchmakingService {}

impl MatchmakingService for DwMatchmakingService {}

impl DwMatchmakingService {
    pub fn new() -> Self {
        DwMatchmakingService {}
    }
}
