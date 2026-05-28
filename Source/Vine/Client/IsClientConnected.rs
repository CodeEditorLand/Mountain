//! Whether the named sidecar currently has a live entry in the
//! connection pool. Cheap read of the shared map; no RPC issued. Useful
//! for boot-race callers that need to know whether `SendRequest` would
//! short-circuit *before* paying the serialization + lock-acquire cost.

pub fn Fn(SideCarIdentifier:&str) -> bool { ::Vine::Client::IsClientConnected::Fn(SideCarIdentifier) }
