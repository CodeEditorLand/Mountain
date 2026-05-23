//! Cocoon → Mountain `window.showTextDocument` notification.
//! Fires when an extension calls
//! `vscode.window.showTextDocument(uri, options)`. Extension activation
//! commonly uses this for "jump to definition" and "reveal config".
//! Forwarded on `sky://window/showTextDocument`.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn WindowShowTextDocument(Service:&MountainVinegRPCService, Parameter:&Value) {
	if let Err(Error) = Service.ApplicationHandle().emit("sky://window/showTextDocument", Parameter) {
		dev_log!(
			"grpc",
			"warn: [MountainVinegRPCService] sky://window/showTextDocument emit failed: {}",
			Error
		);
	}
}
