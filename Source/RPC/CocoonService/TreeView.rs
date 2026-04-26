#![allow(non_snake_case)]
//! Tree View domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: register_tree_view_provider, get_tree_children.

use serde_json::{Value, json};
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::{IPC::SkyEvent::SkyEvent, LanguageFeature::DTO::ProviderType::ProviderType};

use super::CocoonServiceImpl;
use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Vine::{
		Client::SendRequest,
		Generated::{
			Empty,
			GetTreeChildrenRequest,
			GetTreeChildrenResponse,
			RegisterTreeViewProviderRequest,
			TreeItem,
		},
	},
	dev_log,
};

/// Matches the viewId-derived handle used by `RegisterTreeViewProvider`.
fn ViewIdHandle(ViewId:&str) -> u32 {
	ViewId
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32))
}

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

	let Handle = ViewIdHandle(&req.view_id);
	let Provider = Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.GetProvider(Handle);

	if Provider.is_none() {
		dev_log!(
			"tree-view",
			"[TreeView] get-children view={} parent_handle={} - no provider registered",
			req.view_id,
			req.tree_item_handle
		);
		return Ok(Response::new(GetTreeChildrenResponse { items:Vec::new() }));
	}

	dev_log!(
		"tree-view",
		"[TreeView] get-children view={} parent_handle={} - forwarding to Cocoon $provideTreeChildren",
		req.view_id,
		req.tree_item_handle
	);

	// Round-trip to the Cocoon-side TreeDataProvider. The sidecar identifier
	// mirrors the one `RegisterTreeViewProvider` stored. The handler key
	// `$provideTreeChildren` is the VS Code ext-host shim name the shim layer
	// dispatches on; see
	// `Cocoon/Source/Service/ExtensionHostHandler/TreeView.ts` (added in the
	// same batch).
	let Parameters = json!({
		"viewId": req.view_id,
		"treeItemHandle": req.tree_item_handle,
		"handle": Handle,
	});

	// 5s default - TreeDataProvider.getChildren can walk FS for folder views,
	// but should never block a UI thread forever. Longer waits fall through
	// to an empty result (consumer can re-request when the view is focused).
	let Response_ = match SendRequest("cocoon-main", "$provideTreeChildren".to_string(), Parameters, 5000).await {
		Ok(Value_) => Value_,
		Err(Error) => {
			dev_log!(
				"tree-view",
				"[TreeView] get-children view={} error forwarding to Cocoon: {:?}",
				req.view_id,
				Error
			);
			return Ok(Response::new(GetTreeChildrenResponse { items:Vec::new() }));
		},
	};

	let Items = Response_
		.get("items")
		.and_then(Value::as_array)
		.cloned()
		.unwrap_or_default()
		.into_iter()
		.map(|Item| {
			let Handle = Item.get("handle").and_then(Value::as_str).unwrap_or("").to_string();
			let Label = Item.get("label").and_then(Value::as_str).unwrap_or("").to_string();
			let IsCollapsed = Item.get("isCollapsed").and_then(Value::as_bool).unwrap_or(false);
			let Icon = Item.get("icon").and_then(Value::as_str).unwrap_or("").to_string();
			TreeItem { handle:Handle, label:Label, is_collapsed:IsCollapsed, icon:Icon }
		})
		.collect::<Vec<TreeItem>>();

	dev_log!(
		"tree-view",
		"[TreeView] get-children view={} parent_handle={} children={}",
		req.view_id,
		req.tree_item_handle,
		Items.len()
	);

	Ok(Response::new(GetTreeChildrenResponse { items:Items }))
}
