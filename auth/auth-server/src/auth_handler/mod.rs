use crate::response::auth_response::AuthResponse;
use bitdemon::messaging::bd_message::BdMessage;
use bitdemon::networking::bd_session::BdSession;
use num_derive::{FromPrimitive, ToPrimitive};
use std::error::Error;

#[derive(Debug, Eq, PartialEq, Hash, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum AuthMessageType {
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
    fn handle_message(
        &self,
        session: &mut BdSession,
        message: BdMessage,
    ) -> Result<Box<dyn AuthResponse>, Box<dyn Error>>;
}

mod authentication_request;
pub mod steam;
