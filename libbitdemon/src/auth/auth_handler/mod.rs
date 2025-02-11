use crate::auth::response::AuthResponse;
use crate::messaging::bd_message::BdMessage;
use crate::networking::bd_session::BdSession;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::error::Error;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum AuthMessageType {
    CreateAccountRequest = 0x0,
    CreateAccountReply = 0x1,
    ChangeUserKeyRequest = 0x2,
    ChangeUserKeyReply = 0x3,
    ResetAccountRequest = 0x4,
    ResetAccountReply = 0x5,
    DeleteAccountRequest = 0x6,
    DeleteAccountReply = 0x7,
    MigrateAccountsRequest = 0x8,
    MigrateAccountsReply = 0x9,
    AccountForMmpRequest = 0xA,
    AccountForMmpReply = 0xB,
    HostForMmpRequest = 0xC,
    HostForMmpReply = 0xD,
    AccountForHostRequest = 0xE,
    AccountForHostReply = 0xF,
    AnonymousForMmpRequest = 0x10,
    AnonymousForMmpReply = 0x11,
    Ps3ForMmpRequest = 0x12,
    Ps3ForMmpReply = 0x13,
    GetUsernamesByLicenseRequest = 0x14,
    GetUsernamesByLicenseReply = 0x15,
    WiiForMmpRequest = 0x16,
    WiiForMmpReply = 0x17,
    ForDedicatedServerRequest = 0x18,
    ForDedicatedServerReply = 0x19,
    ForDedicatedServerRequestRsa = 0x1A,
    ForDedicatedServerReplyRsa = 0x1B,
    SteamForMmpRequest = 0x1C,
    SteamForMmpReply = 0x1D,
    N3dsForMmpRequest = 0x1E,
    N3dsForMmpReply = 0x1F,
    CodoForMmpRequest = 0x20,
    CodoForMmpReply = 0x21,
    AbaccountsForMmpRequest = 0x22,
    AbaccountsForMmpReply = 0x23,
    WiiUForMmpRequest = 0x24,
    WiiUForMmpReply = 0x25,
    WiiUForMmpRequest2 = 0x26,
    WiiUForMmpReply2 = 0x27,
    WiiUSecondaryForMmpRequest = 0x28,
    WiiUSecondaryForMmpReply = 0x29,
}

impl AuthMessageType {
    pub fn is_request_code(&self) -> bool {
        self.to_u8().unwrap() % 2 == 0
    }

    pub fn reply_code(&self) -> AuthMessageType {
        let code = self.to_u8().unwrap();
        Self::from_u8((code - (code % 2)) + 1).unwrap()
    }
}

pub type ThreadSafeAuthHandler = dyn AuthHandler + Sync + Send;

pub trait AuthHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        message: BdMessage,
    ) -> Result<Box<dyn AuthResponse>, Box<dyn Error>>;
}

mod authentication_request;
pub mod steam;
