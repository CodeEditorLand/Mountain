//! Local error stack - currently unused.
//!
//! Every Mountain consumer uses `CommonLibrary::Error::CommonError`
//! instead. Files remain in place to preserve the original taxonomy;
//! remove or migrate when the strategy is settled.

/// Configurationerror module.
pub mod ConfigurationError;

/// Coreerror module.
pub mod CoreError;

/// Filesystemerror module.
pub mod FileSystemError;

/// Ipcerror module.
pub mod IPCError;

/// Providererror module.
pub mod ProviderError;

/// Serviceerror module.
pub mod ServiceError;
