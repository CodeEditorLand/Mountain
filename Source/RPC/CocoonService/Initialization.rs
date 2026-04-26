#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]
//! Initialization domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: cancel_operation, initial_handshake, init_extension_host.

use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::{
	ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	Vine::Generated::{CancelOperationRequest, Empty, InitExtensionHostRequest},
	dev_log,
};

/// Cancel operations requested by Mountain.
pub async fn CancelOperation(
	Service:&CocoonServiceImpl,
	req:CancelOperationRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Cancel operation request: {}",
		req.request_identifier_to_cancel
	);

	if let Some(Token) = Service.ActiveOperations.read().await.get(&req.request_identifier_to_cancel) {
		dev_log!(
			"cocoon",
			"[CocoonService] Triggering cancellation token for operation {}",
			req.request_identifier_to_cancel
		);
		Token.cancel();
	} else {
		dev_log!(
			"cocoon",
			"warn: [CocoonService] No active operation found for cancellation: {}",
			req.request_identifier_to_cancel
		);
	}

	Ok(Response::new(Empty {}))
}

/// Handshake - Called by Cocoon to signal readiness.
pub async fn InitialHandshake(_Service:&CocoonServiceImpl, _req:Empty) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Initial handshake received from Cocoon");
	Ok(Response::new(Empty {}))
}

/// Initialize Extension Host - Mountain sends initialization data to Cocoon.
pub async fn InitExtensionHost(
	Service:&CocoonServiceImpl,
	req:InitExtensionHostRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Initializing extension host with {} workspace folders",
		req.workspace_folders.len()
	);

	for Folder in &req.workspace_folders {
		dev_log!(
			"cocoon",
			"[CocoonService] Workspace folder: {} ({})",
			Folder.name,
			Folder.uri.as_ref().map(|U| &U.value).unwrap_or(&String::new())
		);
	}

	dev_log!("cocoon", "[CocoonService] Configuration: {} keys", req.configuration.len());

	let Folders:Vec<WorkspaceFolderStateDTO> = req
		.workspace_folders
		.iter()
		.enumerate()
		.filter_map(|(Index, F)| {
			let UriValue = F.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");
			url::Url::parse(UriValue)
				.ok()
				.and_then(|ParsedUrl| WorkspaceFolderStateDTO::New(ParsedUrl, F.name.clone(), Index).ok())
		})
		.collect();

	if !Folders.is_empty() {
		Service.environment.ApplicationState.Workspace.SetWorkspaceFolders(Folders);
		dev_log!(
			"cocoon",
			"[CocoonService] Workspace folders stored: {}",
			req.workspace_folders.len()
		);
	}

	Ok(Response::new(Empty {}))
}
