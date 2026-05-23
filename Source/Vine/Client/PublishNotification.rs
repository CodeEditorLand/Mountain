//! Internal: publish a notification to the broadcast. Called from
//! `SendNotification::Fn` after the wire send succeeds, and from the
//! streaming multiplexer once it lands. `try_send` semantics - no
//! awaiting, no failure surfaced (a slow subscriber must not stall
//! the producer).

use serde_json::Value;

use crate::{
	IPC::DevLog,
	Vine::Client::{NotificationFrame, Shared},
};

pub fn Fn(SideCarIdentifier:&str, Method:&str, Parameters:&Value) {
	let Frame = NotificationFrame::Struct {
		SideCarIdentifier:SideCarIdentifier.to_string(),

		Method:Method.to_string(),

		Parameters:Parameters.clone(),

		TimestampNanos:DevLog::NowNano::Fn(),
	};

	let _ = Shared::NOTIFICATION_BROADCAST.send(Frame);
}
