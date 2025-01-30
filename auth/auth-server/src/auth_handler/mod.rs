use bitdemon::messaging::bd_message::BdMessage;
use bitdemon::networking::bd_session::BdSession;
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Eq, PartialEq, Hash, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum AuthHandlerType {
    CreateAccountRequest,
    CreateAccountReply,
    ChangeUserKeyRequest,
    ChangeUserKeyReply,
    ResetAccountRequest,
    ResetAccountReply,
    DeleteAccountRequest,
    DeleteAccountReply,
    MigrateAccountsRequest,
    MigrateAccountsReply,
    AccountForMmpRequest,
    AccountForMmpReply,
    HostForMmpRequest,
    HostForMmpReply,
    AccountForHostRequest,
    AccountForHostReply,
    AnonymousForMmpRequest,
    AnonymousForMmpReply,
    Ps3ForMmpRequest,
    Ps3ForMmpReply,
    GetUsernamesByLicenseRequest,
    GetUsernamesByLicenseReply,
    WiiForMmpRequest,
    WiiForMmpReply,
    ForDedicatedServerRequest,
    ForDedicatedServerReply,
    ForDedicatedServerRequestRsa,
    ForDedicatedServerReplyRsa,
    SteamForMmpRequest,
    SteamForMmpReply,
    N3dsForMmpRequest,
    N3dsForMmpReply,
    CodoForMmpRequest,
    CodoForMmpReply,
    AbaccountsForMmpRequest,
    AbaccountsForMmpReply,
    WiiUForMmpRequest,
    WiiUForMmpReply,
    WiiUForMmpRequest2,
    WiiUForMmpReply2,
    WiiUSecondaryForMmpRequest,
    WiiUSecondaryForMmpReply,
}

pub trait AuthHandler {
    fn handle_message(&self, session: &mut BdSession, message: BdMessage);
}

pub mod steam;
