#![allow(non_snake_case)]
//! Workspace domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: open_document, save_all, apply_edit,
//! update_configuration, update_workspace_folders.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::{
	ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	Vine::Generated::{
		ApplyEditRequest,
		ApplyEditResponse,
		Empty,
		OpenDocumentRequest,
		OpenDocumentResponse,
		SaveAllRequest,
		SaveAllResponse,
		UpdateConfigurationRequest,
		UpdateWorkspaceFoldersRequest,
	},
	dev_log,
};

pub async fn OpenDocument(
	Service:&CocoonServiceImpl,
	req:OpenDocumentRequest,
) -> Result<Response<OpenDocumentResponse>, Status> {
	let Uri = req.uri.as_ref().map(|U| U.value.clone()).unwrap_or_default();
	dev_log!("cocoon", "[CocoonService] open_document: {}", Uri);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://editor/openDocument",
		json!({ "uri": Uri, "viewColumn": req.view_column }),
	);

	Ok(Response::new(OpenDocumentResponse { success:true }))
}

pub async fn SaveAll(Service:&CocoonServiceImpl, req:SaveAllRequest) -> Result<Response<SaveAllResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] save_all: includeUntitled={}", req.include_untitled);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://editor/saveAll", json!({ "includeUntitled": req.include_untitled }));

	Ok(Response::new(SaveAllResponse { success:true }))
}

pub async fn ApplyEdit(
	Service:&CocoonServiceImpl,
	req:ApplyEditRequest,
) -> Result<Response<ApplyEditResponse>, Status> {
	let Uri = req.uri.as_ref().map(|U| U.value.clone()).unwrap_or_default();
	dev_log!("cocoon", "[CocoonService] apply_edit: uri={} edits={}", Uri, req.edits.len());

	let EditsJson:Vec<serde_json::Value> = req.edits.iter().map(|E| {
		json!({
			"range": {
				"start": E.range.as_ref().and_then(|R| R.start.as_ref()).map(|P| json!({ "line": P.line, "character": P.character })),
				"end": E.range.as_ref().and_then(|R| R.end.as_ref()).map(|P| json!({ "line": P.line, "character": P.character })),
			},
			"newText": E.new_text,
		})
	}).collect();

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://editor/applyEdits", json!({ "uri": Uri, "edits": EditsJson }));

	Ok(Response::new(ApplyEditResponse { success:true }))
}

pub async fn UpdateConfiguration(
	Service:&CocoonServiceImpl,
	req:UpdateConfigurationRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] update_configuration: {} changed keys",
		req.changed_keys.len()
	);

	// Forward configuration changes to Sky for workbench settings refresh
	let _ = Service.environment.ApplicationHandle.emit(
		"sky://configuration/changed",
		json!({
			"changedKeys": req.changed_keys,
		}),
	);

	Ok(Response::new(Empty {}))
}

pub async fn UpdateWorkspaceFolders(
	Service:&CocoonServiceImpl,
	req:UpdateWorkspaceFoldersRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Updating workspace: {} additions, {} removals",
		req.additions.len(),
		req.removals.len()
	);

	// Update WorkspaceState in MountainEnvironment
	for addition in &req.additions {
		dev_log!(
			"cocoon",
			"[CocoonService] Adding workspace folder: {} ({})",
			addition.name,
			addition.uri.as_ref().map(|u| &u.value).unwrap_or(&"?".to_string())
		);
	}
	for removal in &req.removals {
		dev_log!(
			"cocoon",
			"[CocoonService] Removing workspace folder: {}",
			removal.uri.as_ref().map(|u| &u.value).unwrap_or(&"?".to_string())
		);
	}

	// Apply additions and removals to ApplicationState.Workspace
	{
		let mut Folders = Service.environment.ApplicationState.Workspace.GetWorkspaceFolders();

		// Remove by URI
		let RemovalUris:Vec<String> = req
			.removals
			.iter()
			.filter_map(|F| F.uri.as_ref().map(|U| U.value.clone()))
			.collect();
		Folders.retain(|F| !RemovalUris.contains(&F.URI.to_string()));

		// Append additions
		let ExistingCount = Folders.len();
		for (Idx, Addition) in req.additions.iter().enumerate() {
			let UriValue = Addition.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");
			if let Ok(ParsedUrl) = url::Url::parse(UriValue) {
				if let Ok(DTO) = WorkspaceFolderStateDTO::New(ParsedUrl, Addition.name.clone(), ExistingCount + Idx) {
					Folders.push(DTO);
				}
			}
		}

		Service.environment.ApplicationState.Workspace.SetWorkspaceFolders(Folders);
	}

	Ok(Response::new(Empty {}))
}
