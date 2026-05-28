//! Single connection attempt without retry logic. Tunes h2 transport
//! windows for loopback-to-Cocoon traffic (4 MB stream / 16 MB connection)
//! so a single rust-analyzer diagnostic emit (200-500 KB) doesn't cause
//! `WINDOW_UPDATE` ping-pong.
//!
//! On success stores the connected `CocoonClient` in
//! `Shared::SIDECAR_CLIENTS`. If `LAND_VINE_STREAMING=1` is set we also
//! open the bidirectional streaming multiplexer alongside the unary
//! client; failures there are logged and tolerated (Cocoon's streaming
//! handler tree is still on its way).

use crate::Vine::Error::VineError;

pub async fn Fn(SideCarIdentifier:&str, Endpoint:&str) -> Result<(), VineError> {
	::Vine::Client::TryConnectSingle::Fn(SideCarIdentifier, Endpoint).await
}
