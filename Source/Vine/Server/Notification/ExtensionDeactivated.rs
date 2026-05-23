//! Cocoon → Mountain `ExtensionDeactivated` notification.
//! Log-only today - Wind listens on `cocoon:extensionActivated` for the
//! positive half; extensions rarely deactivate at runtime outside
//! uninstall (which fires a separate `sky://extensions/uninstalled`
//! emit). Kept named so the wire method doesn't fall to `notif-drop`.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn ExtensionDeactivated(_Service:&MountainVinegRPCService, Parameter:&Value) {
	dev_log!(
		"grpc",
		"[MountainVinegRPCService] Extension deactivated: {}",
		Parameter.get("extensionId").and_then(Value::as_str).unwrap_or("?")
	);
}
