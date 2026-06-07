//! Await Cocoon's gRPC connection without polling. `GetConnectionNotify`
//! returns a shared `tokio::sync::Notify` that `ConnectToSideCar` fires
//! once the handshake succeeds; `WaitForClientConnection` simply awaits
//! it under `tokio::time::timeout`. If the sidecar is already connected
//! when we enter (typical for the second-and-later callers) `notified()`
//! returns immediately because `notify_waiters` has already fired.
//!
//! `BudgetMilliseconds` remains the hard cap so call sites keep their
//! existing behaviour for the pathological "Cocoon never starts" case.

pub async fn Fn(SideCarIdentifier:&str, BudgetMilliseconds:u64) -> bool {
	::Vine::Client::WaitForClientConnection::Fn(SideCarIdentifier, BudgetMilliseconds).await
}
