//! Shared type definitions for RPC services. Generic `Request` /
//! `Response` envelopes; expand as cross-service types accumulate.
/// Generic request envelope wrapping typed payload data.
pub mod Request;

/// Generic response envelope wrapping typed payload data.
pub mod Response;
