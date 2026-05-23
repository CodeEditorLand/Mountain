//! Cocoon → Mountain `extensionHostMessage` notification.
//! Forwards the extension-host binary protocol reply to Wind via the
//! `cocoon:extensionHostReply` Tauri event. Wind's extension-host bridge
//! consumes these replies to complete pending ext-host RPC calls.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn ExtensionHostMessage(Service:&MountainVinegRPCService, Parameter:&Value) {
	if let Err(Error) = Service.ApplicationHandle().emit("cocoon:extensionHostReply", Parameter) {
		dev_log!(
			"grpc",
			"warn: [MountainVinegRPCService] Failed to emit cocoon:extensionHostReply: {}",
			Error
		);
	}
}
