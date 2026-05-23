
//! Local error stack - currently unused.
//!
//! Every Mountain consumer uses `CommonLibrary::Error::CommonError`
//! instead. Files remain in place to preserve the original taxonomy;
//! remove or migrate when the strategy is settled.

pub mod ConfigurationError;

pub mod CoreError;

pub mod FileSystemError;

pub mod IPCError;

pub mod ProviderError;

pub mod ServiceError;
