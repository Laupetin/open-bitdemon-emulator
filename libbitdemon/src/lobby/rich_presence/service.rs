use crate::networking::bd_session::BdSession;

/// Errors that may occur when handling storage calls.
#[derive(Debug)]
pub enum RichPresenceServiceError {
    /// The authenticated user does not have permission to perform the requested operation.
    PermissionDeniedError,
    /// The rich presence data is too long to process.
    RichPresenceDataTooLargeError,
    /// Requested rich presence data for too many users
    TooManyUsersError,
}

pub type ThreadSafeRichPresenceService = dyn RichPresenceService + Sync + Send;

/// Implements domain logic concerning rich presence.
pub trait RichPresenceService {
    /// Sets rich presence for the current session.
    fn set_info(
        &self,
        session: &BdSession,
        user_id: u64,
        rich_presence_data: Vec<u8>,
    ) -> Result<(), RichPresenceServiceError>;

    /// Retrieves the rich presence for the specified group of users.
    /// Results for users are returned in the same order as requested.
    fn get_info(
        &self,
        session: &BdSession,
        users: &[u64],
    ) -> Result<Vec<Option<Vec<u8>>>, RichPresenceServiceError>;
}
