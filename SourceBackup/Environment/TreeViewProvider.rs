// @module TreeViewProvider (Environment)
// @description Implements the `TreeViewProvider` trait for
// `MountainEnvironment` by delegating to the logic Handler in
// `Handler::tree_view`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{
	Environment::Requires,
	error::CommonError,
	tree_view::{TreeViewProvider, DTO::*},
};
use serde_json::Value;

use super::MountainEnvironment;
use crate::Handler::tree_view as TreeViewHandler;

#[async_trait]
impl TreeViewProvider for MountainEnvironment {
	// Handle registering a new tree data provider by delegating to the
	// `TreeViewHandler`.
	async fn RegisterTreeDataProvider(&self, view_id:String, options:TreeViewOptionsDTO) -> Result<(), CommonError> {
		TreeViewHandler::RegisterTreeDataProviderLogic(&self.ApplicationHandle, view_id, options).await
	}

	// Handle unregistering a tree data provider by delegating to the
	// `TreeViewHandler`.
	async fn UnregisterTreeDataProvider(&self, view_id:String) -> Result<(), CommonError> {
		TreeViewHandler::UnregisterTreeDataProviderLogic(&self.ApplicationHandle, view_id).await
	}

	// Handle revealing a tree item by delegating to the `TreeViewHandler`.
	async fn RevealTreeItem(
		&self,
		view_id:String,
		item:TreeItemDTO,
		parent_chain:Vec<TreeItemDTO>,
		options:RevealOptionsDTO,
	) -> Result<(), CommonError> {
		TreeViewHandler::RevealTreeItemLogic(&self.ApplicationHandle, view_id, item, parent_chain, options).await
	}

	// Handle refreshing a tree view by delegating to the `TreeViewHandler`.
	async fn RefreshTreeView(&self, view_id:String, items_to_refresh:Option<Value>) -> Result<(), CommonError> {
		TreeViewHandler::RefreshTreeViewLogic(&self.ApplicationHandle, view_id, items_to_refresh).await
	}

	// Handle setting a tree view's message by delegating to the
	// `TreeViewHandler`.
	async fn SetTreeViewMessage(&self, view_id:String, message:Value) -> Result<(), CommonError> {
		TreeViewHandler::SetTreeViewMessageLogic(&self.ApplicationHandle, view_id, message).await
	}

	// Handle setting a tree view's title by delegating to the
	// `TreeViewHandler`.
	async fn SetTreeViewTitle(
		&self,
		view_id:String,
		title:String,
		description:Option<String>,
	) -> Result<(), CommonError> {
		TreeViewHandler::SetTreeViewTitleLogic(&self.ApplicationHandle, view_id, title, description).await
	}

	// Handle setting a tree view's badge by delegating to the
	// `TreeViewHandler`.
	async fn SetTreeViewBadge(&self, view_id:String, badge:Option<TreeViewBadgeDTO>) -> Result<(), CommonError> {
		TreeViewHandler::SetTreeViewBadgeLogic(&self.ApplicationHandle, view_id, badge).await
	}
}

impl Requires<Arc<dyn TreeViewProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TreeViewProvider + Send + Sync> { Arc::new(self.clone()) }
}
