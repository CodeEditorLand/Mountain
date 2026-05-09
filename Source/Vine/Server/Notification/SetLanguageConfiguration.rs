#![allow(non_snake_case)]
//! Cocoon → Mountain `set_language_configuration` notification.
//! Emitted by `Cocoon/.../APIFactoryService.ts:557` when an extension
//! calls `vscode.languages.setLanguageConfiguration(languageId, config)`.
//! Carries brackets / indent rules / word-pattern / comments. Forwards on
//! `sky://language/configure`; Monaco's config side reads the payload and
//! calls `monaco.languages.setLanguageConfiguration(...)`.

use serde_json::Value;

use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn SetLanguageConfiguration(Service:&MountainVinegRPCService, Parameter:&Value) {

	let _ = Service.ApplicationHandle().emit("sky://language/configure", Parameter);

	dev_log!(
		"grpc",

		"[Language] configure id={}",

		Parameter.get("languageId").and_then(Value::as_str).unwrap_or("?")
	);
}
