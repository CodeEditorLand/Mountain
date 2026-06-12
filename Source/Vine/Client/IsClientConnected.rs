//! Checks whether a named sidecar currently has a live entry in the
//! connection pool. Performs a cheap read of the shared map; no RPC issued.
//! Useful for boot-race callers that need to know whether `SendRequest` would
//! short-circuit *before* paying the serialization plus lock-acquire cost.

/// Public entry point for this module.
pub fn Fn(SideCarIdentifier:&str) -> bool { ::Vine::Client::IsClientConnected::Fn(SideCarIdentifier) }
