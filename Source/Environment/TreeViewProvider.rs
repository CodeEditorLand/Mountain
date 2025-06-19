//! # TreeViewProvider Implementation
//!
//! Implements the `TreeViewProvider` trait for the `MountainEnvironment`. This
//! provider manages the lifecycle of custom tree views and orchestrates the
//! data flow between the extension host (`Cocoon`) and the UI (`Sky`).

use Common::{Error::CommonError, TreeView::TreeViewProvider};
use async_trait::async_trait;
use log::{info, warn};
use serde_json::{Value, json};
use tauri::Emitter;

use super::{MountainEnvironment, Utility};
use crate::ApplicationState::DTO::TreeViewStateDTO;

#[async_trait]
impl TreeViewProvider for MountainEnvironment {
	/// Registers a new tree data provider from Cocoon.
	async fn RegisterTreeDataProvider(&self, ViewIdentifier:String, Options:Value) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Registering data provider for view: {}", ViewIdentifier);
		let OptionsDTO:crate::Common::TreeView::DTO::TreeViewOptionsDTO = serde_json::from_value(Options)
			.map_err(|e| CommonError::InvalidArgument { ArgumentName:"Options".into(), Reason:e.to_string() })?;

		let NewState = TreeViewStateDTO {
			ViewIdentifier:ViewIdentifier.clone(),
			CanSelectMany:OptionsDTO.CanSelectMany,
			HasHandleDrag:OptionsDTO.HasHandleDrag,
			HasHandleDrop:OptionsDTO.HasHandleDrop,
		};

		self.ApplicationState
			.ActiveTreeViews
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.insert(ViewIdentifier.clone(), NewState);

		// Notify the frontend to create the UI for this tree view.
		self.ApplicationHandle
			.emit(
				"sky://tree-view/create",
				json!({ "ViewIdentifier": ViewIdentifier, "Options": OptionsDTO }),
			)
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })
	}

	async fn UnregisterTreeDataProvider(&self, ViewIdentifier:String) -> Result<(), CommonError> {
		info!("[TreeViewProvider] Unregistering data provider for view: {}", ViewIdentifier);
		self.ApplicationState.ActiveTreeViews.lock().unwrap().remove(&ViewIdentifier);
		self.ApplicationHandle
			.emit("sky://tree-view/dispose", json!({ "ViewIdentifier": ViewIdentifier }))
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })
	}

	async fn RevealTreeItem(
		&self,
		_ViewIdentifier:String,
		_Item:Value,
		_ParentChain:Vec<Value>,
		_Options:Value,
	) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] RevealTreeItem is not implemented.");
		Ok(())
	}

	async fn RefreshTreeView(&self, _ViewIdentifier:String, _ItemsToRefresh:Option<Value>) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] RefreshTreeView is not implemented.");
		Ok(())
	}

	async fn SetTreeViewMessage(&self, _ViewIdentifier:String, _Message:Value) -> Result<(), CommonError> {
		warn!("[TreeViewProvider] SetTreeViewMessage is not implemented.");
		Ok(())
	}

	async fn SetTreeViewTitle(
		&self,
		_ViewIdentifier:String,
		_Title:String,
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
