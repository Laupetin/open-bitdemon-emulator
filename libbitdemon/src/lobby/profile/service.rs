use crate::networking::bd_session::BdSession;

/// Errors that may occur when handling storage calls.
#[derive(Debug)]
pub enum ProfileServiceError {
    /// The authenticated user does not have permission to perform the requested operation.
    PermissionDenied,
    /// The requested profile could not be found.
    NoProfileInfoFound,
}

/// Represents the profile info that a client set as a blob.
pub struct ProfileInfo {
    /// The id of the user that this profile information is from.
    pub user_id: u64,
    /// The opaque profile data that the user set.
    pub data: Vec<u8>,
}

pub type ThreadSafeProfileService = dyn ProfileService + Sync + Send;

/// Implements domain logic concerning profiles.
pub trait ProfileService {
    /// Retrieves the public profile info for the specified users.
    fn get_public_profiles(
        &self,
        session: &BdSession,
        user_ids: Vec<u64>,
    ) -> Result<Vec<ProfileInfo>, ProfileServiceError>;

    /// Retrieves the private profile info for the current authenticated user.
    fn get_private_profile(&self, session: &BdSession) -> Result<ProfileInfo, ProfileServiceError>;

    /// Sets the public profile info for the current authenticated user.
    fn set_public_profile(
        &self,
        session: &BdSession,
        public_profile_data: Vec<u8>,
    ) -> Result<(), ProfileServiceError>;

    /// Sets the private profile info for the current authenticated user.
    fn set_private_profile(
        &self,
        session: &BdSession,
        private_profile_data: Vec<u8>,
    ) -> Result<(), ProfileServiceError>;

    /// Removes all profile information for the current authenticated user.
    fn delete_profile(&self, session: &BdSession) -> Result<(), ProfileServiceError>;
}
