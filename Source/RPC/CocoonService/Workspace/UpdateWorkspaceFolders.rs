//! Apply folder additions/removals to `ApplicationState.Workspace`. URIs
//! drive removal matching; new folders are appended after the existing
//! set so existing indices stay stable.

use tonic::{Response, Status};

use crate::{
	ApplicationState::{
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		Struct::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndNotify,
	},
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, UpdateWorkspaceFoldersRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:UpdateWorkspaceFoldersRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Updating workspace: {} additions, {} removals",
		Request.additions.len(),
		Request.removals.len()
	);

	for Addition in &Request.additions {
		dev_log!(
			"cocoon",
			"[CocoonService] Adding workspace folder: {} ({})",
			Addition.name,
			Addition.uri.as_ref().map(|U| &U.value).unwrap_or(&"?".to_string())
		);
	}

	for Removal in &Request.removals {
		dev_log!(
			"cocoon",
			"[CocoonService] Removing workspace folder: {}",
			Removal.uri.as_ref().map(|U| &U.value).unwrap_or(&"?".to_string())
		);
	}

	{
		let mut Folders = Service.environment.ApplicationState.Workspace.GetWorkspaceFolders();

		let RemovalURIs:Vec<String> = Request
			.removals
			.iter()
			.filter_map(|F| F.uri.as_ref().map(|U| U.value.clone()))
			.collect();

		Folders.retain(|F| !RemovalURIs.contains(&F.URI.to_string()));

		let ExistingCount = Folders.len();

		for (Index, Addition) in Request.additions.iter().enumerate() {
			let URI = Addition.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

			if let Ok(Parsed) = url::Url::parse(URI) {
				if let Ok(DTO) = WorkspaceFolderStateDTO::New(Parsed, Addition.name.clone(), ExistingCount + Index) {
					Folders.push(DTO);
				}
			}
		}

		UpdateWorkspaceFoldersAndNotify(&Service.environment.ApplicationState.Workspace, Folders);
	}

	Ok(Response::new(Empty {}))
}
