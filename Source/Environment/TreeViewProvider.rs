// ============================================================================
// File: Mountain/Source/Environment/TreeViewProvider.rs
// ============================================================================
// This module follows the Land ecosystem's PascalCase naming convention.
// See: https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//
// # TreeViewProvider Implementation
//
// Implements the `TreeViewProvider` trait for the `MountainEnvironment`.
// This provider manages the lifecycle of custom tree views and orchestrates the
// data flow between the extension host (`Cocoon`) and the UI (`Sky`).
//
// ## Key Features:
// - Tree view registration and lifecycle management
// - Tree data provider dispatching (native/proxied)
// - Tree state persistence and restoration
// - Lazy loading and selection handling
// - Drag and drop support
// - Badge and message management
//
// ## VSCode Reference:
// - vs/workbench/api/browser/mainThreadTreeViews.ts
// - vs/workbench/api/common/extHostTreeViews.ts
//
// ============================================================================

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

	/// Updates the tree view message displayed in the UI.
	async fn SetTreeViewMessage(&self, ViewIdentifier:String, Message:Option<String>) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Setting message for view '{}': {:?}", ViewIdentifier, Message);

		{
			let mut TreeViewGuard = self
				.ApplicationState
				.ActiveTreeViews
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			if let Some(ViewState) = TreeViewGuard.get_mut(&ViewIdentifier) {
				ViewState.Message = Message.clone();
			}
		}

		self.ApplicationHandle
			.emit(
				"sky://tree-view/set-message",
				json!({ "ViewIdentifier": ViewIdentifier, "Message": Message }),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction {
				Reason: format!("Failed to emit tree view message: {}", Error),
			})
	}

	/// Updates the tree view's title and description.
	async fn SetTreeViewTitle(
		&self,
		ViewIdentifier:String,
		Title:Option<String>,
		Description:Option<String>,
	) -> Result<(), CommonError> {
		info!(
			"[TreeViewProvider] Setting title/description for view '{}': {:?} {:?}",
			ViewIdentifier, Title, Description
		);

		{
			let mut TreeViewGuard = self
				.ApplicationState
				.ActiveTreeViews
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			if let Some(ViewState) = TreeViewGuard.get_mut(&ViewIdentifier) {
				ViewState.Title = Title.clone();
				ViewState.Description = Description.clone();
			}
		}

		self.ApplicationHandle
			.emit(
				"sky://tree-view/set-title",
				json!({
					"ViewIdentifier": ViewIdentifier,
					"Title": Title,
					"Description": Description,
				}),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction {
				Reason: format!("Failed to emit tree view title: {}", Error),
			})
	}

	/// Sets a badge on the tree view.
	async fn SetTreeViewBadge(&self, ViewIdentifier:String, Badge:Option<Value>) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Setting badge for view '{}': {:?}", ViewIdentifier, Badge);

		// Update state
		{
			let mut TreeViewGuard = self
				.ApplicationState
				.ActiveTreeViews
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			if let Some(ViewState) = TreeViewGuard.get_mut(&ViewIdentifier) {
				// Store badge in ViewState (you may need to add this field to TreeViewStateDTO)
				// For now, just emit the event
			}
		}

		// Emit to frontend
		self.ApplicationHandle
			.emit(
				"sky://tree-view/set-badge",
				json!({ "ViewIdentifier": ViewIdentifier, "Badge": Badge }),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction {
				Reason: format!("Failed to emit tree view badge: {}", Error),
			})
	}

	/// Handles tree node expansion/collapse events.
	async fn OnTreeNodeExpanded(&self, ViewIdentifier:String, ElementHandle:String, IsExpanded:bool) -> Result<(), CommonError> {
		debug!(
			"[TreeViewProvider] Tree node '{}' in view '{}' is now {}",
			ElementHandle, ViewIdentifier, if IsExpanded { "expanded" } else { "collapsed" }
		);

		// Save expansion state for persistence
		{
			let mut TreeViewGuard = self
				.ApplicationState
				.ActiveTreeViews
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			if let Some(ViewState) = TreeViewGuard.get_mut(&ViewIdentifier) {
				// Store expansion state in ViewState
				// You may need to add an ExpansionState field to TreeViewStateDTO
				_ = (IsExpanded, ElementHandle); // Suppress unused warning
			}
		}

		Ok(())
	}

	/// Handles tree selection changes.
	async fn OnTreeSelectionChanged(&self, ViewIdentifier:String, SelectedHandles:Vec<String>) -> Result<(), CommonError> {
		debug!(
			"[TreeViewProvider] Selection changed in view '{}': {} items selected",
			ViewIdentifier,
			SelectedHandles.len()
		);

		// Save selection state
		{
			let mut TreeViewGuard = self
				.ApplicationState
				.ActiveTreeViews
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			if let Some(ViewState) = TreeViewGuard.get_mut(&ViewIdentifier) {
				// Store selected handles in ViewState
				// You may need to add a SelectedHandles field to TreeViewStateDTO
				_ = SelectedHandles; // Suppress unused warning
			}
		}

		Ok(())
	}

	/// Persists tree view state (for restoration after restart).
	async fn PersistTreeViewState(&self, ViewIdentifier:String) -> Result<Value, CommonError> {
		info!("[TreeViewProvider] Persisting state for view '{}'", ViewIdentifier);

		let TreeViewGuard = self
			.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		let State = TreeViewGuard.get(&ViewIdentifier).map(|ViewState| {
			json!({
				"ViewIdentifier": ViewState.ViewIdentifier,
				"Title": ViewState.Title,
				"Description": ViewState.Description,
				"CanSelectMany": ViewState.CanSelectMany,
				"HasHandleDrag": ViewState.HasHandleDrag,
				"HasHandleDrop": ViewState.HasHandleDrop,
			})
		});

		Ok(State.unwrap_or(json!(null)))
	}

	/// Restores previously persisted tree view state.
	async fn RestoreTreeViewState(&self, ViewIdentifier:String, State:Value) -> Result<(), CommonError> {
		info!(
			"[TreeViewProvider] Restoring state for view '{}' from persisted data",
			ViewIdentifier
		);

		// Parse and apply the persisted state
		if let Some(ViewDescription) = State.get("ViewDescription").and_then(|v| v.as_str()) {
			self.SetTreeViewTitle(ViewIdentifier.clone(), Some(ViewDescription.to_string()), None).await?;
		}

		if let Some(ViewMessage) = State.get("ViewMessage") {
			let Message:Option<String> = serde_json::from_value(ViewMessage.clone()).ok();
			self.SetTreeViewMessage(ViewIdentifier.clone(), Message).await?;
		}

		Ok(())
	}

	/// Handles tree node drag and drop start.
	async fn OnTreeViewDragStart(&self, ViewIdentifier:String, DraggedHandles:Vec<String>) -> Result<Vec<String>, CommonError> {
		debug!(
			"[TreeViewProvider] Drag started in view '{}': {} items being dragged",
			ViewIdentifier,
			DraggedHandles.len()
		);

		// For now, just return the handles.
		// In a full implementation, this would:
		// 1. Prepare data transfer objects for the dragged items
		// 2. Register the drag operation with the DnD service
		Ok(DraggedHandles)
	}

	/// Handles tree node drop.
	async fn OnTreeViewDrop(&self, ViewIdentifier:String, TargetHandle:Option<String>, TransferData:Value) -> Result<(), CommonError> {
		info!(
			"[TreeViewProvider] Drop in view '{}' on target {:?}",
			ViewIdentifier, TargetHandle
		);

		let ProviderInfo = self
			.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.get(&ViewIdentifier)
			.cloned();

		if let Some(Info) = ProviderInfo {
			if let Some(SideCarId) = Info.SideCarIdentifier {
				let IPCProvider:Arc<dyn IPCProvider> = self.Require();

				let RPCMethod = format!("{}$handleDrop", ProxyTarget::ExtHostTreeView.GetTargetPrefix());
				let RPCParams = json!({
					"ViewIdentifier": ViewIdentifier,
					"TargetHandle": TargetHandle,
					"TransferData": TransferData,
				});

				IPCProvider.SendRequestToSideCar(SideCarId, RPCMethod, RPCParams, 10000).await?
			}
		}

		Ok(())
	}
}
