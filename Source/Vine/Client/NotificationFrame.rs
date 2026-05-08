#![allow(non_snake_case)]

//! One observed notification frame fanned out from `SendNotification`
//! (or, once the streaming-channel multiplexer is live, from
//! `Multiplexer`). Subscribers consume frames from the broadcast channel
//! managed by `Shared::NOTIFICATION_BROADCAST`.

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Struct {
	pub SideCarIdentifier:String,

	pub Method:String,

	pub Parameters:Value,

	/// Monotonic process-relative nanosecond timestamp at fan-out time.
	/// Useful for OTel span correlation without burning a
	/// `SystemTime::now()` per frame.
	pub TimestampNanos:u64,
}
