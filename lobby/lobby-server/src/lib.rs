mod response;

use crate::response::task_reply::TaskReply;
use bitdemon::messaging::bd_message::BdMessage;
use bitdemon::messaging::bd_response::{BdResponse, ResponseCreator};
use bitdemon::messaging::BdErrorCode::ServiceNotAvailable;
use bitdemon::networking::bd_session::BdSession;
use bitdemon::networking::bd_socket::BdMessageHandler;
use log::warn;
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
    RichPresenceService = 68,
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

pub trait LobbyHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>>;
}

pub struct LobbyServer {
    lobby_handlers: RwLock<HashMap<LobbyServiceId, Arc<dyn LobbyHandler + Sync + Send>>>,
}

impl LobbyServer {
    pub fn new() -> Self {
        let handlers: HashMap<LobbyServiceId, Arc<dyn LobbyHandler + Sync + Send>> = HashMap::new();

        // handlers.insert(
        //     AuthMessageType::SteamForMmpRequest,
        //     Arc::new(SteamAuthHandler::new()),
        // );

        LobbyServer {
            lobby_handlers: RwLock::new(handlers),
        }
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
        let service_id_input = message.reader.read_u8()?;

        let service_id = LobbyServiceId::from_u8(service_id_input)
            .ok_or_else(|| IllegalServiceIdSnafu { service_id_input }.build())?;

        let handlers = self.lobby_handlers.read().unwrap();
        let maybe_handler = handlers.get(&service_id);

        match maybe_handler {
            Some(handler) => {
                let response = handler.handle_message(session, message)?;
                response.send(session)?;

                Ok(())
            }
            None => {
                warn!(
                    "[Session {}] Tried to call unavailable service {service_id:?}",
                    session.id
                );
                TaskReply::with_only_error_code(ServiceNotAvailable)
                    .to_response()?
                    .send(session)?;

                Ok(())
            }
        }
    }
}
