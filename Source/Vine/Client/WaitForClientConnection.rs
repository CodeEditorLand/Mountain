#![allow(non_snake_case)]

//! Await Cocoon's gRPC connection without polling. `GetConnectionNotify`
//! returns a shared `tokio::sync::Notify` that `ConnectToSideCar` fires
//! once the handshake succeeds; `WaitForClientConnection` simply awaits
//! it under `tokio::time::timeout`. If the sidecar is already connected
//! when we enter (typical for the second-and-later callers) `notified()`
//! returns immediately because `notify_waiters` has already fired.
//!
//! `BudgetMilliseconds` remains the hard cap so call sites keep their
//! existing behaviour for the pathological "Cocoon never starts" case.

use std::time::Duration;

use crate::Vine::Client::{IsClientConnected, IsShuttingDown, Shared::GetConnectionNotify};

pub async fn Fn(SideCarIdentifier:&str, BudgetMilliseconds:u64) -> bool {
	if IsShuttingDown::Fn() {
		return false;
	}

	if IsClientConnected::Fn(SideCarIdentifier) {
		return true;
	}

	let Notifier = GetConnectionNotify(SideCarIdentifier);

	let Result = tokio::time::timeout(Duration::from_millis(BudgetMilliseconds), Notifier.notified()).await;

	if Result.is_err() {
		// Budget expired - do a final check in case the connection landed
		// in the window between notified() registering and the timeout.
		return IsClientConnected::Fn(SideCarIdentifier);
	}

	// Woken by FireConnectionNotify - the client is now in the pool.
	true
}
