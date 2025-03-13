mod content_streaming;
mod counter;
mod group;
mod profile;
mod rich_presence;
mod storage;

use crate::config::DwServerConfig;
use crate::lobby::content_streaming::create_content_streaming_handler;
use crate::lobby::counter::create_counter_handler;
use crate::lobby::group::create_group_handler;
use crate::lobby::profile::create_profile_handler;
use crate::lobby::rich_presence::create_rich_presence_handler;
use crate::lobby::storage::create_storage_handler;
use axum::Router;
use bitdemon::lobby::anti_cheat::AntiCheatHandler;
use bitdemon::lobby::bandwidth::BandwidthHandler;
use bitdemon::lobby::dml::DmlHandler;
use bitdemon::lobby::league::LeagueHandler;
use bitdemon::lobby::title_utilities::TitleUtilitiesHandler;
use bitdemon::lobby::vote_rank::VoteRankHandler;
use bitdemon::lobby::LobbyServiceId::{
    Anticheat, BandwidthTest, Counter, Dml, Group, League, Profile, RichPresence, Storage,
    TitleUtilities, VoteRank,
};
use bitdemon::lobby::{LobbyServer, LobbyServiceId, ThreadSafeLobbyHandler};
use bitdemon::networking::session_manager::SessionManager;
use std::cell::Cell;
use std::sync::Arc;

pub fn configure_lobby_server(
    lobby_server: &LobbyServer,
    session_manager: Arc<SessionManager>,
    config: &DwServerConfig,
) -> Router {
    let mut configurer = DwServerConfigurer::new(lobby_server);

    configurer.direct_config(Anticheat, Arc::new(AntiCheatHandler::new()));
    configurer.direct_config(BandwidthTest, Arc::new(BandwidthHandler::new()));

    configurer.full_config(create_content_streaming_handler(config));

    configurer.direct_config(Counter, create_counter_handler());
    configurer.direct_config(Dml, Arc::new(DmlHandler::new()));
    configurer.direct_config(Group, create_group_handler(session_manager.clone()));
    configurer.direct_config(League, Arc::new(LeagueHandler::new()));
    configurer.direct_config(Profile, create_profile_handler());
    configurer.direct_config(RichPresence, create_rich_presence_handler(session_manager));
    configurer.direct_config(Storage, create_storage_handler());
    configurer.direct_config(TitleUtilities, Arc::new(TitleUtilitiesHandler::new()));
    configurer.direct_config(VoteRank, Arc::new(VoteRankHandler::new()));

    configurer.into()
}

pub struct ConfiguredEnvironment {
    service_id: LobbyServiceId,
    handler: Arc<ThreadSafeLobbyHandler>,
    pub_router: Option<Router>,
}

impl ConfiguredEnvironment {
    pub fn new(
        service_id: LobbyServiceId,
        handler: Arc<ThreadSafeLobbyHandler>,
    ) -> ConfiguredEnvironment {
        ConfiguredEnvironment {
            service_id,
            handler,
            pub_router: None,
        }
    }

    pub fn with_pub_router(mut self, router: Router) -> Self {
        self.pub_router = Some(router);

        self
    }

    pub fn configure_lobby_server(self, lobby_server: &LobbyServer) {
        lobby_server.add_service(self.service_id, self.handler);
    }

    pub fn configure_pub_router(&mut self, mut pub_router: Router) -> Router {
        if let Some(self_router) = self.pub_router.take() {
            pub_router = pub_router.merge(self_router);
        }

        pub_router
    }
}

struct DwServerConfigurer<'a> {
    lobby_server: &'a LobbyServer,
    pub_router: Cell<Router>,
}

impl<'a> DwServerConfigurer<'a> {
    fn new(lobby_server: &'a LobbyServer) -> Self {
        DwServerConfigurer {
            lobby_server,
            pub_router: Cell::new(Router::new()),
        }
    }

    fn direct_config(
        &self,
        lobby_service_id: LobbyServiceId,
        handler: Arc<ThreadSafeLobbyHandler>,
    ) {
        self.lobby_server.add_service(lobby_service_id, handler);
    }

    fn full_config(&mut self, mut env: ConfiguredEnvironment) {
        self.pub_router
            .set(env.configure_pub_router(self.pub_router.take()));
        env.configure_lobby_server(self.lobby_server)
    }
}

impl<'a> From<DwServerConfigurer<'a>> for Router {
    fn from(value: DwServerConfigurer<'a>) -> Self {
        value.pub_router.take()
    }
}
