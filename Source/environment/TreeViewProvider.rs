use std::sync::Arc;

use Common::{
	environment::Requires,
	error::CommonError,
	tree_view::{TreeViewProvider, dto::*},
};
use async_trait::async_trait;
use serde_json::Value;

/// @module TreeViewProvider (Environment)
/// @description Implements the `TreeViewProvider` trait for
/// `MountainEnvironment` by delegating to the logic handlers in
/// `handlers::tree_view`.
use super::MountainEnvironment;
use crate::handlers::tree_view as TreeViewHandler;

#[async_trait]
impl TreeViewProvider for MountainEnvironment {
	/// Handles registering a new tree data provider by delegating to the
	/// `TreeViewHandler`.
	async fn RegisterTreeDataProvider(&self, ViewId:String, Options:TreeViewOptionsDto) -> Result<(), CommonError> {
		TreeViewHandler::RegisterTreeDataProviderLogic(&self.AppHandle, ViewId, Options).await
	}

	/// Handles unregistering a tree data provider by delegating to the
	/// `TreeViewHandler`.
	async fn UnregisterTreeDataProvider(&self, ViewId:String) -> Result<(), CommonError> {
		TreeViewHandler::UnregisterTreeDataProviderLogic(&self.AppHandle, ViewId).await
	}

	/// Handles revealing a tree item by delegating to the `TreeViewHandler`.
	async fn RevealTreeItem(
		&self,
		ViewId:String,
		Item:TreeItemDto,
		ParentChain:Vec<TreeItemDto>,
		Options:RevealOptionsDto,
	) -> Result<(), CommonError> {
		TreeViewHandler::RevealTreeItemLogic(&self.AppHandle, ViewId, Item, ParentChain, Options).await
	}

	/// Handles refreshing a tree view by delegating to the `TreeViewHandler`.
	async fn RefreshTreeView(&self, ViewId:String, ItemsToRefresh:Option<Value>) -> Result<(), CommonError> {
		TreeViewHandler::RefreshTreeViewLogic(&self.AppHandle, ViewId, ItemsToRefresh).await
	}

	/// Handles setting a tree view's message by delegating to the
	/// `TreeViewHandler`.
	async fn SetTreeViewMessage(&self, ViewId:String, Message:Value) -> Result<(), CommonError> {
		// A real implementation would delegate to a handler.
		// For now, this remains a no-op stub.
		Ok(())
	}

	/// Handles setting a tree view's title by delegating to the
	/// `TreeViewHandler`.
	async fn SetTreeViewTitle(
		&self,
		ViewId:String,
		Title:String,
		Description:Option<String>,
	) -> Result<(), CommonError> {
		// A real implementation would delegate to a handler.
		Ok(())
	}

	/// Handles setting a tree view's badge by delegating to the
	/// `TreeViewHandler`.
	async fn SetTreeViewBadge(&self, ViewId:String, Badge:Option<TreeViewBadgeDto>) -> Result<(), CommonError> {
		// A real implementation would delegate to a handler.
		Ok(())
	}
}

impl Requires<Arc<dyn TreeViewProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TreeViewProvider + Send + Sync> { Arc::new(self.clone()) }
}
