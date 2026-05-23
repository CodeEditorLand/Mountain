
//! Round-trip a `getChildren` request to the Cocoon-side
//! `TreeDataProvider` over Vine. Returns an empty list when no provider
//! is registered or the sidecar call times out (5 s default).

use serde_json::{Value, json};
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::{CocoonServiceImpl, TreeView::ViewIdHandle},
	Vine::{
		Client::SendRequest::Fn as SendRequest,
		Generated::{GetTreeChildrenRequest, GetTreeChildrenResponse, TreeItem},
	},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:GetTreeChildrenRequest,
) -> Result<Response<GetTreeChildrenResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] get_tree_children: view={}", Request.view_id);

	let Handle = ViewIdHandle::Fn(&Request.view_id);

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
			Request.view_id,
			Request.tree_item_handle
		);

		return Ok(Response::new(GetTreeChildrenResponse { items:Vec::new() }));
	}

	dev_log!(
		"tree-view",
		"[TreeView] get-children view={} parent_handle={} - forwarding to Cocoon $provideTreeChildren",
		Request.view_id,
		Request.tree_item_handle
	);

	let Parameters = json!({
		"viewId": Request.view_id,
		"treeItemHandle": Request.tree_item_handle,
		"handle": Handle,
	});

	let Reply = match SendRequest("cocoon-main", "$provideTreeChildren".to_string(), Parameters, 5000).await {
		Ok(Value_) => Value_,

		Err(Error) => {
			dev_log!(
				"tree-view",
				"[TreeView] get-children view={} error forwarding to Cocoon: {:?}",
				Request.view_id,
				Error
			);

			return Ok(Response::new(GetTreeChildrenResponse { items:Vec::new() }));
		},
	};

	let Items = Reply
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
		Request.view_id,
		Request.tree_item_handle,
		Items.len()
	);

	Ok(Response::new(GetTreeChildrenResponse { items:Items }))
}
