#![allow(non_snake_case)]
//! Cocoon → Mountain `ExtensionActivated` notification.
//! Fires once per extension when its `activate` export resolves (or
//! finishes registering contributions). Forwarded to Wind on
//! `cocoon:extensionActivated` so the Extensions sidebar updates its
//! row state without polling.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn ExtensionActivated(Service:&MountainVinegRPCService, Parameter:&Value) {
	if let Err(Error) = Service.ApplicationHandle().emit("cocoon:extensionActivated", Parameter) {
		dev_log!(
			"grpc",
			"warn: [MountainVinegRPCService] Failed to emit cocoon:extensionActivated: {}",
			Error
		);
	}
}
