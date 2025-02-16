pub mod anti_cheat;
pub mod bandwidth;
pub mod counter;
pub mod dml;
pub mod group;
pub mod league;
mod lsg;
mod response;
pub mod rich_presence;
pub mod storage;
pub mod title_utilities;

use crate::auth::key_store::ThreadSafeBackendPrivateKeyStorage;
use crate::lobby::lsg::LsgHandler;
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyServiceId::LobbyService;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::BdErrorCode::{AccessDenied, ServiceNotAvailable};
use crate::networking::bd_session::BdSession;
use crate::networking::bd_socket::BdMessageHandler;
use log::{info, warn};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use snafu::Snafu;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum LobbyServiceId {
    Teams = 3,
    Stats = 4,
    Messaging = 6,
    LobbyService = 7,
    Profiles = 8,
    Friends = 9,
    Storage = 10,
    Messaging2 = 11,
    TitleUtilities = 12,
    KeyArchive = 15,
    BandwidthTest = 18,
    Stats2 = 19,
    Matchmaking = 21,
    Stats3 = 22,
    Counter = 23,
    // 26 = ? Some references to license and auth; 3 task ids
    Dml = 27,
    Group = 28,
    Mail = 29,
    Twitch = 31,
    Youtube = 33,
    Twitter = 35,
    Facebook = 36,
    Anticheat = 38,
    ContentStreaming = 50,
    Tags = 52,
    // 53 = ?
    VoteRank = 55,
    LinkCode = 57,
    PooledStorage = 58,
    Subscription = 66,
    EventLog = 67,
    RichPresence = 68,
    League = 81,
    League2 = 82,
    // Services with unknown IDs:
    // UCD
    // - IsRegistered
    // - CreateAccount
    // - GetUserDetails
    // - GetUserDetailsByEmail
    // - AuthorizeGuestUser
    // - AuthorizeGuestUserByEmail
    // - UpdateUserDetails
    // - UpdateMarketingOptIn
    //
    // ContentUnlock
    // - ListContentByLicenseCode
    // - ListContentByLicenseCodeWithSubtype
    // - ListContent
    // - ListContentWithSubtype
    // - UnlockContentByLicenseCode
    // - UnlockContentByLicenseCodeWithSubtype
    // - UnlockSharedContentByLicenseCode
    // - UnlockSharedContentByLicenseCodeWithSubtype
    // - UnlockContent
    // - UnlockContentWithSubtype
    // - UnlockSharedContent
    // - UnlockSharedContentWithSubtype
    // - ListUnlockedContent
    // - ListUnlockedContentWithSubtype
    // - ListUnlockedSharedContent
    // - ListUnlockedSharedContentWithSubtype
    // - CheckContentStatusByLicenseCodes
    // - TakeOwnershipOfUsersSharedContent
    // - SynchronizeUnlockedContent
    //
    // UserGroups
    // - CreateGroup
    // - DeleteGroup
    // - JoinGroup
    // - LeaveGroup
    // - GetMembershipInfo
    // - ChangeMemberType
    // - GetNumMembers
    // - GetMembers
    // - GetMemberships
    // - GetGroupLists
    // - ReadStatsByRank
    //
    // Marketplace
    // - GetBalance
    // - Deposit
    // - GetProducts
    // - GetSkus
    // - PurchaseSkus
    // - GetInventory
    // - PutInventoryItem
    // - PutPlayersInventoryItems
    // - ConsumeInventoryItem
    // - ConsumeInventoryItems
    // - GetPlayersInventories
    // - DeleteInventory
    // - PutPlayersEntitlements
    // - GetPlayersEntitlements
    //
    // Commerce
    // - GetBalances
    // - Deposit
    // - ModifyBalances
    // - SetBalances
    // - MigrateBalances
    // - SetWriter
    // - GetWriter
    // - GetWriters
    // - GetLastWriter
    // - ValidateReceipt
    // - GetItems
    // - GetGiftsOfferedToUser
    // - GetGiftsOfferedByUser
    // - RetractGiftOffers
    // - AcceptGifts
    // - RejectGifts
    // - PurchaseItems
    // - ConsumeItems
    // - GiftItems
    // - SetInventory
    // - SetItems
    // - SetItemQuantities
    // - TransferInventory
    // - ConsolidateItems
    //
    // FeatureBan
    // - GetFeatureBans
    //
    // Tencent
    // - VerifyString
    // - SanitizeString
    // - GetAASRecord
    // - GetAASRecordsByUserID
    // - RegisterCodoID
    //
    // FacebookLite
    // - RegisterAccount
    // - RegisterToken
    // - Post
    // - UnregisterAccount
    // - UploadPhoto
    // - IsRegistered
    // - GetInfo
    // - GetRegisteredAccounts
    //
    // CRUX
    // - RegisterAndAuthorize
    // - Authorize
    //
    // PresenceService
    // - SetPresenceData
    // - GetPresenceData
    //
    // RelayService
    // - GetCredentials
    //
    // LinkedAccounts
    // - GetDataIdentifiers
    // - GetLinkedAccounts
    // - SwitchContextData
}

pub type ThreadSafeLobbyHandler = dyn LobbyHandler + Sync + Send;

pub trait LobbyHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>>;

    fn requires_authentication(&self) -> bool {
        true
    }
}

pub struct LobbyServer {
    lobby_handlers: RwLock<HashMap<LobbyServiceId, Arc<ThreadSafeLobbyHandler>>>,
}

impl LobbyServer {
    pub fn new(key_store: Arc<ThreadSafeBackendPrivateKeyStorage>) -> Self {
        let lobby_server = LobbyServer {
            lobby_handlers: RwLock::new(HashMap::new()),
        };

        lobby_server.add_service(LobbyService, Arc::new(LsgHandler::new(key_store)));

        lobby_server
    }

    pub fn add_service(&self, service_id: LobbyServiceId, handler: Arc<ThreadSafeLobbyHandler>) {
        info!("Adding {service_id:?} lobby handler");
        self.lobby_handlers
            .write()
            .unwrap()
            .insert(service_id, handler);
    }
}

#[derive(Debug, Snafu)]
enum LobbyServerError {
    #[snafu(display("The client specified an illegal service id: {service_id_input}"))]
    IllegalServiceIdError { service_id_input: u8 },
}

impl BdMessageHandler for LobbyServer {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<(), Box<dyn Error>> {
        message.reader.set_type_checked(false);
        let service_id_input = message.reader.read_u8()?;

        let service_id = LobbyServiceId::from_u8(service_id_input)
            .ok_or_else(|| IllegalServiceIdSnafu { service_id_input }.build())?;

        let handlers = self.lobby_handlers.read().unwrap();
        let maybe_handler = handlers.get(&service_id);

        match maybe_handler {
            Some(handler) => {
                if handler.requires_authentication() && session.authentication().is_none() {
                    warn!("Tried to service {service_id:?} that requires authentication while being unauthenticated");
                    TaskReply::with_only_error_code(AccessDenied, 0)
                        .to_response()?
                        .send(session)?;
                } else {
                    message.reader.set_type_checked(true);
                    let mut response = handler.handle_message(session, message)?;
                    response.send(session)?;
                }

                Ok(())
            }
            None => {
                warn!("Tried to call unavailable service {service_id:?}");
                TaskReply::with_only_error_code(ServiceNotAvailable, 0)
                    .to_response()?
                    .send(session)?;

                Ok(())
            }
        }
    }
}
