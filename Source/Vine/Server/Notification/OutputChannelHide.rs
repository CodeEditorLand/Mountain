//! Cocoon → Mountain `outputChannel.hide` notification.
//! Forwards to Sky as `sky://output/show { visible: false, channel }` so
//! the workbench panel can dismiss the channel via the same handler that
//! processes `show()` calls. Stock VS Code's `OutputChannel.hide()` is
//! the counterpart of `show()` and extensions expect the panel to stop
//! displaying the channel when invoked.

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn OutputChannelHide(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Channel = Parameter
		.get("channel")
		.or_else(|| Parameter.get("name"))
		.or_else(|| Parameter.get("handle"))
		.and_then(Value::as_str)
		.unwrap_or("");

	let _ = Service.ApplicationHandle().emit(
		"sky://output/show",
		json!({
			"visible": false,
			"channel": Channel,
		}),
	);

	dev_log!("grpc", "[OutputChannel] hide channel={}", Channel);
}
