#![allow(non_snake_case)]
//! Cocoon → Mountain `workspace.applyEdit` notification.
//! Fires when an extension calls `vscode.workspace.applyEdit(edit)`
//! with a multi-file `WorkspaceEdit`. The payload shape matches VS
//! Code's `IWorkspaceEdit`; Sky's BulkEditService applies the edits
//! against open models.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn WorkspaceApplyEdit(Service:&MountainVinegRPCService, Parameter:&Value) {
	if let Err(Error) = Service.ApplicationHandle().emit("sky://workspace/applyEdit", Parameter) {
		dev_log!(
			"grpc",
			"warn: [MountainVinegRPCService] sky://workspace/applyEdit emit failed: {}",
			Error
		);
	}
}
