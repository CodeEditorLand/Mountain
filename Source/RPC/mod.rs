//! # RPC
//!
//! Mountain RPC services. `CocoonService` is the active surface — the tonic
//! server implementation that the extension host connects to.

/// CocoonService: tonic gRPC server exposing the full extension-host API
/// surface.
pub mod CocoonService;
