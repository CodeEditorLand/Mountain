#![allow(non_snake_case)]
//! Tree View domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: register_tree_view_provider, get_tree_children.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::{IPC::SkyEvent::SkyEvent, LanguageFeature::DTO::ProviderType::ProviderType};

use super::CocoonServiceImpl;
use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Vine::Generated::{Empty, GetTreeChildrenRequest, GetTreeChildrenResponse, RegisterTreeViewProviderRequest},
	dev_log,
};

pub async fn RegisterTreeViewProvider(
	Service:&CocoonServiceImpl,
	req:RegisterTreeViewProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering tree view provider: {}", req.view_id);

	let Handle = req
		.view_id
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));
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

	// Emit on the canonical `SkyEvent::TreeViewCreate` channel so the
	// renderer's SkyBridge listener (and every downstream `cel:tree-view`
	// consumer) picks it up via the same path used by the generic
	// `tree.register` effect. The previous `sky://treeView/register`
	// channel was a parallel fork that no listener ever subscribed to.
	let _ = Service.environment.ApplicationHandle.emit(
		SkyEvent::TreeViewCreate.AsStr(),
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
	// For now return empty - will be wired when Cocoon activation is complete.
	Ok(Response::new(GetTreeChildrenResponse { items:Vec::new() }))
}
