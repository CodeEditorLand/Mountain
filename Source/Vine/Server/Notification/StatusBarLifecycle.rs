//! Cocoon → Mountain `statusBar.update` / `statusBar.dispose` notifications.
//! Each `vscode.window.createStatusBarItem(...)` instance fires
//! `statusBar.update` with text / tooltip / alignment; `statusBar.dispose`
//! removes the item. Sky's workbench status-bar renderer subscribes to
//! the downstream `sky://statusbar/*` family.
//!
//! Canonical channel prefix is `sky://statusbar/` (no hyphen) to match
//! every other emit site in the statusbar group; the legacy
//! `sky://status-bar/*` fork has been retired.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn StatusBarLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	let EventName = format!("sky://statusbar/{}", &MethodName["statusBar.".len()..]);

	if let Err(Error) = Service.ApplicationHandle().emit(&EventName, Parameter) {
		dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
	}
}
