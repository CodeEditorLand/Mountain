// File: Mountain/Source/Environment/TreeViewProvider.rs

//! # TreeViewProvider Implementation
//!
//! Implements the `TreeViewProvider` trait for the `MountainEnvironment`. This
//! provider manages the lifecycle of custom tree views and orchestrates the
//! data flow between the extension host (`Cocoon`) and the UI (`Sky`).

use Common::{
	Error::CommonError::CommonError,
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
	/// Registers a new tree data provider from Cocoon (an extension).
	async fn RegisterTreeDataProvider(&self, ViewIdentifier:String, Options:Value) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Registering data provider for view: {}", ViewIdentifier);
		let OptionsDTO:TreeViewOptionsDTO = serde_json::from_value(Options)
			.map_err(|e| CommonError::InvalidArgument { ArgumentName:"Options".into(), Reason:e.to_string() })?;

		let NewState = TreeViewStateDTO {
			ViewIdentifier:ViewIdentifier.clone(),
			Provider:None,
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
				json!({ "ViewIdentifier": ViewIdentifier, "Options": OptionsDTO }),
			)
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;

		Ok(())
	}

	/// Unregisters a tree data provider from Cocoon.
	async fn UnregisterTreeDataProvider(&self, ViewIdentifier:String) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Unregistering data provider for view: {}", ViewIdentifier);
		self.ApplicationState.ActiveTreeViews.lock().unwrap().remove(&ViewIdentifier);
		self.ApplicationHandle
			.emit("sky://tree-view/dispose", json!({ "ViewIdentifier": ViewIdentifier }))
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })
	}

	/// Reveals a specific item in the tree view.
	async fn RevealTreeItem(
		&self,
		_ViewIdentifier:String,
		_ItemHandle:String,
		_Options:Value,
	) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] RevealTreeItem is not implemented.");
		Ok(())
	}

	/// Refreshes the tree view, optionally starting from a specific set of
	/// items.
	async fn RefreshTreeView(&self, _ViewIdentifier:String, _ItemsToRefresh:Option<Value>) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] RefreshTreeView is not implemented.");
		Ok(())
	}

	/// Sets a message to be displayed in the tree view UI.
	async fn SetTreeViewMessage(&self, _ViewIdentifier:String, _Message:Option<String>) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] SetTreeViewMessage is not implemented.");
		Ok(())
	}

	/// Sets the title and description for the tree view.
	async fn SetTreeViewTitle(
		&self,
		_ViewIdentifier:String,
		_Title:Option<String>,
		_Description:Option<String>,
	) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] SetTreeViewTitle is not implemented.");
		Ok(())
	}

	/// Sets a badge to be displayed on the tree view's container.
	async fn SetTreeViewBadge(&self, _ViewIdentifier:String, _Badge:Option<Value>) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] SetTreeViewBadge is not implemented.");
		Ok(())
	}

	/// Gets the children for a given element. This method acts as a dispatcher.
	async fn GetChildren(
		&self,
		ViewIdentifier:String,
		ElementHandle:Option<String>,
	) -> Result<Vec<Value>, CommonError> {
		let provider = self
			.ApplicationState
			.ActiveTreeViews
			.lock()
			.unwrap()
			.get(&ViewIdentifier)
			.and_then(|v| v.Provider.clone());

		if let Some(p) = provider {
			p.GetChildren(ViewIdentifier, ElementHandle).await
		} else {
			Err(CommonError::NotImplemented { FeatureName:"GetChildren for proxied TreeView".into() })
		}
	}

	/// Gets the TreeItem for a given element. This method acts as a dispatcher.
	async fn GetTreeItem(&self, ViewIdentifier:String, ElementHandle:String) -> Result<Value, CommonError> {
		let provider = self
			.ApplicationState
			.ActiveTreeViews
			.lock()
			.unwrap()
			.get(&ViewIdentifier)
			.and_then(|v| v.Provider.clone());

		if let Some(p) = provider {
			p.GetTreeItem(ViewIdentifier, ElementHandle).await
		} else {
			Err(CommonError::NotImplemented { FeatureName:"GetTreeItem for proxied TreeView".into() })
		}
	}
}
