// File: Mountain/Source/Environment/TreeViewProvider.rs

//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # TreeViewProvider Implementation
//!
//! Implements the `TreeViewProvider` trait for the `MountainEnvironment`. This
//! provider manages the lifecycle of custom tree views and orchestrates the
//! data flow between the extension host (`Cocoon`) and the UI (`Sky`).

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
	TreeView::{DTO::TreeViewOptionsDTO::TreeViewOptionsDTO, TreeViewProvider::TreeViewProvider},
};
use async_trait::async_trait;
use log::{info, warn};
use serde_json::{Value, json};
use tauri::Emitter;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::TreeViewStateDTO::TreeViewStateDTO;

#[async_trait]
impl TreeViewProvider for MountainEnvironment {
	/// Registers a new tree data provider from an extension (e.g., Cocoon).
	async fn RegisterTreeDataProvider(&self, ViewIdentifier:String, Options:Value) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Registering data provider for view: {}", ViewIdentifier);

		let OptionsDTO:TreeViewOptionsDTO = serde_json::from_value(Options.clone()).map_err(|Error| {
			CommonError::InvalidArgument { ArgumentName:"Options".into(), Reason:Error.to_string() }
		})?;

		// For now, assume all extension providers come from the main sidecar.
		let SideCarIdentifier = "cocoon-main".to_string();

		let NewState = TreeViewStateDTO {
			ViewIdentifier:ViewIdentifier.clone(),

			// This is a proxied provider, not native.
			Provider:None,

			SideCarIdentifier:Some(SideCarIdentifier),

			CanSelectMany:OptionsDTO.CanSelectMany,

			HasHandleDrag:OptionsDTO.HasHandleDrag,

			HasHandleDrop:OptionsDTO.HasHandleDrop,

			Message:None,

			Title:None,

			Description:None,
		};

		self.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.insert(ViewIdentifier.clone(), NewState);

		self.ApplicationHandle
			.emit(
				"sky://tree-view/create",
				json!({ "ViewIdentifier": ViewIdentifier, "Options": Options }),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;

		Ok(())
	}

	/// Unregisters a tree data provider.
	async fn UnregisterTreeDataProvider(&self, ViewIdentifier:String) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Unregistering data provider for view: {}", ViewIdentifier);

		self.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&ViewIdentifier);

		self.ApplicationHandle
			.emit("sky://tree-view/dispose", json!({ "ViewIdentifier": ViewIdentifier }))
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Reveals a specific item in the tree view by notifying the UI.
	async fn RevealTreeItem(&self, ViewIdentifier:String, ItemHandle:String, Options:Value) -> Result<(), CommonError> {
		info!(
			"[TreeViewProvider] Revealing item '{}' in view '{}'",
			ItemHandle, ViewIdentifier
		);

		self.ApplicationHandle
			.emit(
				"sky://tree-view/reveal",
				json!({ "viewId": ViewIdentifier, "itemHandle": ItemHandle, "options": Options }),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Refreshes the tree view by notifying the UI.
	async fn RefreshTreeView(&self, ViewIdentifier:String, ItemsToRefresh:Option<Value>) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Refreshing view '{}'", ViewIdentifier);

		self.ApplicationHandle
			.emit(
				"sky://tree-view/refresh",
				json!({ "viewId": ViewIdentifier, "itemsToRefresh": ItemsToRefresh }),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Gets the children for a given element. This method acts as a dispatcher.
	async fn GetChildren(
		&self,

		ViewIdentifier:String,

		ElementHandle:Option<String>,
	) -> Result<Vec<Value>, CommonError> {
		let ProviderInfo = self
			.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.get(&ViewIdentifier)
			.cloned();

		if let Some(Info) = ProviderInfo {
			if let Some(NativeProvider) = Info.Provider {
				// Case 1: Native Rust provider (e.g., File Explorer)
				return NativeProvider.GetChildren(ViewIdentifier, ElementHandle).await;
			} else if let Some(SideCarId) = Info.SideCarIdentifier {
				// Case 2: Proxied extension provider
				let IPCProvider:Arc<dyn IPCProvider> = self.Require();

				let RPCMethod = format!("{}$getChildren", ProxyTarget::ExtHostTreeView.GetTargetPrefix());

				let RPCParams = json!([ViewIdentifier, ElementHandle]);

				let Response = IPCProvider.SendRequestToSideCar(SideCarId, RPCMethod, RPCParams, 10000).await?;

				return serde_json::from_value::<Vec<Value>>(Response).map_err(CommonError::from);
			}
		}
		Err(CommonError::TreeViewProviderNotFound { ViewIdentifier })
	}

	/// Gets the TreeItem for a given element. This method acts as a dispatcher.
	async fn GetTreeItem(&self, ViewIdentifier:String, ElementHandle:String) -> Result<Value, CommonError> {
		let ProviderInfo = self
			.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.get(&ViewIdentifier)
			.cloned();

		if let Some(Info) = ProviderInfo {
			if let Some(NativeProvider) = Info.Provider {
				return NativeProvider.GetTreeItem(ViewIdentifier, ElementHandle).await;
			} else if let Some(SideCarId) = Info.SideCarIdentifier {
				let IPCProvider:Arc<dyn IPCProvider> = self.Require();

				let RPCMethod = format!("{}$getTreeItem", ProxyTarget::ExtHostTreeView.GetTargetPrefix());

				let RPCParams = json!([ViewIdentifier, ElementHandle]);

				return IPCProvider.SendRequestToSideCar(SideCarId, RPCMethod, RPCParams, 5000).await;
			}
		}
		Err(CommonError::TreeViewProviderNotFound { ViewIdentifier })
	}

	// --- Other stubbed methods ---
	async fn SetTreeViewMessage(&self, _ViewIdentifier:String, _Message:Option<String>) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] SetTreeViewMessage is not implemented.");

		Ok(())
	}

	async fn SetTreeViewTitle(
		&self,

		_ViewIdentifier:String,

		_Title:Option<String>,

		_Description:Option<String>,
	) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] SetTreeViewTitle is not implemented.");

		Ok(())
	}

	async fn SetTreeViewBadge(&self, _ViewIdentifier:String, _Badge:Option<Value>) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] SetTreeViewBadge is not implemented.");

		Ok(())
	}
}
