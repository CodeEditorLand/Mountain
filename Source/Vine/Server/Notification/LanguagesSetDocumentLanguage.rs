//! Cocoon → Mountain `languages.setDocumentLanguage` notification.
//! Emitted when an extension calls
//! `vscode.languages.setTextDocumentLanguage(document, languageId)`.
//! Forwarded verbatim to Sky on `sky://languages/setDocumentLanguage`
//! so Monaco swaps the language mode on the matching editor.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn Fn(Service:&MountainVinegRPCService, Parameter:&Value) {
	if let Err(Error) = Service
		.ApplicationHandle()
		.emit("sky://languages/setDocumentLanguage", Parameter)
	{
		dev_log!(
			"grpc",
			"warn: [MountainVinegRPCService] sky://languages/setDocumentLanguage emit failed: {}",
			Error
		);
	}
}
