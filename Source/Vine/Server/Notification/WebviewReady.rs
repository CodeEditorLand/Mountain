//! Cocoon → Mountain `WebviewReady` notification.
//! Fires when a webview the extension owns has finished loading its
//! entry HTML. Log-only today - Sky's webview shim handles the DOM-side
//! readiness independently. Kept named so the wire method is
//! observable and doesn't fall to `notif-drop`.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn WebviewReady(_Service:&MountainVinegRPCService, Parameter:&Value) {
	dev_log!(
		"grpc",
		"[MountainVinegRPCService] Webview ready: handle={}",
		Parameter.get("handle").and_then(Value::as_str).unwrap_or("?")
	);
}
