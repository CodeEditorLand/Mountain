//! Whether the named sidecar currently has a live entry in the
//! connection pool. Cheap read of the shared map; no RPC issued. Useful
//! for boot-race callers that need to know whether `SendRequest` would
//! short-circuit *before* paying the serialization + lock-acquire cost.

use crate::Vine::Client::{IsShuttingDown, Shared::SIDECAR_CLIENTS};

pub fn Fn(SideCarIdentifier:&str) -> bool {
	if IsShuttingDown::Fn() {
		return false;
	}

	let Pool = SIDECAR_CLIENTS.lock();

	Pool.contains_key(SideCarIdentifier)
}
