//! # Tree View Data Access Helpers
//!
//! Internal helper functions for fetching tree data (children, tree items).

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
};
use serde_json::Value;

use crate::{Environment::Utility, dev_log};

/// Gets the children for a given element.
/// Acts as a dispatcher to native or extension providers.
pub(super) async fn get_children(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,

	view_identifier:String,

	element_handle:Option<String>,
) -> Result<Vec<Value>, CommonError> {
	dev_log!(
		"extensions",
		"[TreeViewProvider] Getting children for view '{}', handle: {:?}",
		view_identifier,
		element_handle
	);

	let provider_info = env
		.ApplicationState
		.Feature
		.TreeViews
		.ActiveTreeViews
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
		.Get(&view_identifier)
		.cloned();

	if let Some(info) = provider_info {
		if let Some(native_provider) = info.Provider {
			// Case 1: Native Rust provider (e.g., File Explorer)
			return native_provider.GetChildren(view_identifier, element_handle).await;
		} else if let Some(side_car_id) = info.SideCarIdentifier {
			// Case 2: Proxied extension provider
			let ipc_provider:Arc<dyn IPCProvider> = env.Require();

			let rpc_method = format!("{}$getChildren", ProxyTarget::ExtHostTreeView.GetTargetPrefix());

			let RpcParams = serde_json::json!([view_identifier, element_handle]);

			let Response = ipc_provider
				.SendRequestToSideCar(side_car_id, rpc_method, RpcParams, 10000)
				.await?;

			return serde_json::from_value::<Vec<Value>>(response).map_err(CommonError::from);
		}
	}

	Err(CommonError::TreeViewProviderNotFound { ViewIdentifier:view_identifier })
}

/// Gets the TreeItem for a given element.
/// Acts as a dispatcher to native or extension providers.
pub(super) async fn get_tree_item(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,

	view_identifier:String,

	element_handle:String,
) -> Result<Value, CommonError> {
	dev_log!(
		"extensions",
		"[TreeViewProvider] Getting item for view '{}', handle: {}",
		view_identifier,
		element_handle
	);

	let provider_info = env
		.ApplicationState
		.Feature
		.TreeViews
		.ActiveTreeViews
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
		.Get(&view_identifier)
		.cloned();

	if let Some(info) = provider_info {
		if let Some(native_provider) = info.Provider {
			return native_provider.GetTreeItem(view_identifier, element_handle).await;
		} else if let Some(side_car_id) = info.SideCarIdentifier {
			let ipc_provider:Arc<dyn IPCProvider> = env.Require();

			let rpc_method = format!("{}$getTreeItem", ProxyTarget::ExtHostTreeView.GetTargetPrefix());

			let RpcParams = serde_json::json!([view_identifier, element_handle]);

			return ipc_provider
				.SendRequestToSideCar(side_car_id, rpc_method, RpcParams, 5000)
				.await;
		}
	}

	Err(CommonError::TreeViewProviderNotFound { ViewIdentifier:view_identifier })
}
