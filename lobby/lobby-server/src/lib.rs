use bitdemon::messaging::bd_message::BdMessage;
use bitdemon::messaging::bd_response::BdResponse;
use bitdemon::networking::bd_session::BdSession;
use bitdemon::networking::bd_socket::BdMessageHandler;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

#[derive(Debug, Eq, PartialEq, Hash, FromPrimitive, ToPrimitive)]
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
        let mut handlers: HashMap<LobbyServiceId, Arc<dyn LobbyHandler + Sync + Send>> =
            HashMap::new();

        // handlers.insert(
        //     AuthMessageType::SteamForMmpRequest,
        //     Arc::new(SteamAuthHandler::new()),
        // );

        LobbyServer {
            lobby_handlers: RwLock::new(handlers),
        }
    }
}

impl BdMessageHandler for LobbyServer {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<(), Box<dyn Error>> {
        let a = message.reader.read_u8()?;

        let handler_type = LobbyServiceId::from_u8(a).unwrap();

        let handlers = self.lobby_handlers.read().unwrap();
        let handler = handlers.get(&handler_type).unwrap();

        let response = handler.handle_message(session, message)?;
        response.send(session)?;

        Ok(())
    }
}
