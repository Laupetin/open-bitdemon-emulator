/// Errors that may occur when handling matchmaking calls.
#[derive(Debug)]
pub enum MatchmakingServiceError {
    /// The authenticated user does not have permission to perform the requested operation.
    PermissionDenied,
}

pub type ThreadSafeMatchmakingService = dyn MatchmakingService + Sync + Send;

/// Implements domain logic concerning matchmaking.
pub trait MatchmakingService {}
