#![allow(non_snake_case)]

//! Local error stack. **Currently dead code** - every Mountain consumer uses
//! `CommonLibrary::Error::CommonError` instead. Files remain in place to
//! preserve the original taxonomy; remove or migrate when the strategy is
//! settled.
//!
//! TODO: zero callers as of 2026-05-02. Either delete and rely on
//! `CommonLibrary::Error`, or migrate consumers off `CommonError` and onto
//! these richer per-domain types.

pub mod ConfigurationError;
pub mod CoreError;
pub mod FileSystemError;
pub mod IPCError;
pub mod ProviderError;
pub mod ServiceError;
