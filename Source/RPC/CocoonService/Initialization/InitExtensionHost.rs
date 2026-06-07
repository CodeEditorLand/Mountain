//! Mountain → Cocoon initialization payload (workspace folders +
//! configuration). Stores parsed folders into `ApplicationState.Workspace`
//! so the rest of the boot path sees them.

use tonic::{Response, Status};

use ::Vine::Generated::{Empty, InitExtensionHostRequest};

use crate::{
	ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:InitExtensionHostRequest) -> Result<Response<Empty>, Status> {

	dev_log!(
		"cocoon",

		"[CocoonService] Initializing extension host with {} workspace folders",

		Request.workspace_folders.len()
	);

	for Folder in &Request.workspace_folders {
		dev_log!(
			"cocoon",

			"[CocoonService] Workspace folder: {} ({})",

			Folder.name,

			Folder.uri.as_ref().map(|U| &U.value).unwrap_or(&String::new())
		);
	}

	dev_log!("cocoon", "[CocoonService] Configuration: {} keys", Request.configuration.len());

	let Folders:Vec<WorkspaceFolderStateDTO> = Request
		.workspace_folders
		.iter()
		.enumerate()
		.filter_map(|(Index, F)| {
			let URI = F.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

			url::Url::parse(URI)
				.ok()
				.and_then(|Parsed| WorkspaceFolderStateDTO::New(Parsed, F.name.clone(), Index).ok())
		})
		.collect();

	if !Folders.is_empty() {
		Service.environment.ApplicationState.Workspace.SetWorkspaceFolders(Folders);

		dev_log!(
			"cocoon",

			"[CocoonService] Workspace folders stored: {}",

			Request.workspace_folders.len()
		);
	}

	Ok(Response::new(Empty {}))
}
