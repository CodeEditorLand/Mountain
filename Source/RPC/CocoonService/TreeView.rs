#![allow(non_snake_case)]
//! Tree View domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: register_tree_view_provider, get_tree_children.

use std::{
	sync::{
		Arc, Mutex, OnceLock,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
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

/// Coalesced Mountain → Sky emit buffer for `sky://tree-view/create`.
///
/// Each extension that contributes a tree view (Explorer, SCM, Debug,
/// Run, Outline, plus extension-contributed views like Roo, Claude,
/// gitlens) calls `RegisterTreeViewProvider` separately during
/// activation. 30+ emits in a tight burst at boot saturate the Tauri
/// channel that also delivers keystrokes. SkyBridge's listener
/// already accepts both single `{ viewId, extensionId }` and batch
/// `{ views: [{ viewId, extensionId }, ...] }` shapes (mirrors the
/// command-batch pattern from `Vine/Server/Notification/RegisterCommand.rs`).
struct TreeViewEmitBatch {
	Pending:Mutex<Vec<Value>>,
	FlushScheduled:AtomicBool,
}

static TREE_VIEW_EMIT_BATCH:OnceLock<Arc<TreeViewEmitBatch>> = OnceLock::new();

fn EnqueueTreeViewEmit(Handle:&AppHandle, Payload:Value) {
	let Batch = TREE_VIEW_EMIT_BATCH.get_or_init(|| {
		Arc::new(TreeViewEmitBatch { Pending:Mutex::new(Vec::new()), FlushScheduled:AtomicBool::new(false) })
	});

	{
		let mut Pending = Batch.Pending.lock().unwrap();
		Pending.push(Payload);
	}

	if !Batch.FlushScheduled.swap(true, Ordering::AcqRel) {
		let BatchClone = Batch.clone();
		let HandleClone = Handle.clone();
		let Channel = SkyEvent::TreeViewCreate.AsStr().to_string();
		tokio::spawn(async move {
			tokio::time::sleep(Duration::from_millis(16)).await;
			let Drained:Vec<Value> = {
				let mut Pending = BatchClone.Pending.lock().unwrap();
				std::mem::take(&mut *Pending)
			};
			BatchClone.FlushScheduled.store(false, Ordering::Release);
			if Drained.is_empty() {
				return;
			}
			let Count = Drained.len();
			match HandleClone.emit(&Channel, json!({ "views": Drained })) {
				Ok(()) => dev_log!("sky-emit", "[SkyEmit] ok channel={} batch={}", Channel, Count),
				Err(Error) => dev_log!("sky-emit", "[SkyEmit] fail channel={} batch={} error={}", Channel, Count, Error),
			}
		});
	}
}

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

	// Coalesce the Sky emit. SkyBridge's `sky://tree-view/create`
	// listener accepts either single `{ viewId, extensionId }`
	// (legacy / runtime path) or batch `{ views: [{ viewId,
	// extensionId }, ...] }` (extension-boot path). 30+ tree-view
	// registrations during boot collapse to one Tauri emit per
	// 16ms window.
	EnqueueTreeViewEmit(
		&Service.environment.ApplicationHandle,
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
