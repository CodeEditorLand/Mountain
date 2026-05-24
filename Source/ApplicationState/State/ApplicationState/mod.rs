//! `ApplicationState` - atomized.

pub mod MapLockError;
pub mod MapLockErrorWithRecovery;

pub use MapLockError::Fn as MapLockError;
pub use MapLockErrorWithRecovery::Fn as MapLockErrorWithRecovery;
