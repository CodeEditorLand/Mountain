#![allow(non_snake_case)]
//! Tree View domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: register_tree_view_provider, get_tree_children.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;
use crate::dev_log;
use crate::Vine::Generated::{
	Empty, GetTreeChildrenRequest, GetTreeChildrenResponse,
	RegisterTreeViewProviderRequest,
};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

pub async fn RegisterTreeViewProvider(
	Service:&CocoonServiceImpl,
	req:RegisterTreeViewProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering tree view provider: {}", req.view_id);

	let Handle = req.view_id.as_bytes().iter().fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));
	let dto = ProviderRegistrationDTO {
		Handle,
		ProviderType:ProviderType::TreeView,
		Selector:json!([{ "viewId": req.view_id }]),
		SideCarIdentifier:"cocoon-main".to_string(),
		ExtensionIdentifier:json!(req.extension_id),
		Options:Some(json!({ "viewId": req.view_id })),
	};
	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, dto);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://treeView/register",
		json!({ "viewId": req.view_id, "extensionId": req.extension_id }),
	);

	Ok(Response::new(Empty {}))
}

pub async fn GetTreeChildren(
	Service:&CocoonServiceImpl,
	req:GetTreeChildrenRequest,
) -> Result<Response<GetTreeChildrenResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] get_tree_children: view={}", req.view_id);

	// Tree children are fetched by forwarding to Cocoon via the generic RPC path.
	// The extension registers a TreeDataProvider; when Sky needs children,
	// Mountain looks up the provider handle and invokes Cocoon.
	// For now return empty — will be wired when Cocoon activation is complete.
	Ok(Response::new(GetTreeChildrenResponse { items:Vec::new() }))
}
