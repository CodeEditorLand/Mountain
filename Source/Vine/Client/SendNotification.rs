//! Fire-and-forget notification to a sidecar. No response, no per-call
//! timeout. Prefers the streaming multiplexer under
//! `LAND_VINE_STREAMING=1`; falls through to unary on any failure. After
//! a successful wire send, fans out via `PublishNotification` so
//! broadcast subscribers (Effect-TS fibers, OTel emitters, future
//! Mist-WS bridge, dev log) can observe the same flow concurrently.

use serde_json::Value;

use crate::Vine::Error::VineError;

pub async fn Fn(SideCarIdentifier:String, Method:String, Parameters:Value) -> Result<(), VineError> {

	::Vine::Client::SendNotification::Fn(SideCarIdentifier, Method, Parameters).await
}
