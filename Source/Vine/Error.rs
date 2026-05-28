//! # VineError - canonical error surface for the Vine gRPC IPC layer.
//!
//! The variant set, `From` conversions, [`VineError::IsRecoverable`] /
//! [`VineError::ToTonicStatus`] helpers, and the [`Result`] alias all live
//! in [`::Vine::Error`]. This module exposes them at
//! `crate::Vine::Error::*` via type aliases so existing in-tree imports
//! continue to resolve without an explicit workspace-crate import at each
//! call site.

/// Canonical Vine error enum. See [`::Vine::Error::VineError`] for the full
/// variant list and conversion impls.
pub type VineError = ::Vine::Error::VineError;

/// `Result<T, VineError>` convenience alias.
pub type Result<T> = ::Vine::Error::Result<T>;
