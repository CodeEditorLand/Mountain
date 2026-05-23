//! Cocoon → Mountain `outputChannel.hide` notification.
//! Stock VS Code exposes `OutputChannel.hide()` as a counterpart to
//! `show()`. Sky doesn't yet render a dismissable panel per-channel, so
//! the signal currently no-ops to a tagged log line. Kept explicit so
//! the wire name doesn't fall through to `notif-drop` on every
//! extension-driven hide.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputChannelHide(_Service:&MountainVinegRPCService, Parameter:&Value) {
	dev_log!(
		"grpc",
		"[OutputChannel] hide channel={} (no-op - panel dismiss not wired)",
		Parameter.get("channel").and_then(Value::as_str).unwrap_or("?")
	);
}
