//! # TreeViewProvider (Environment)
//!
//! Implements the
//! [`TreeViewProvider`](CommonLibrary::TreeView::TreeViewProvider) trait for
//! `MountainEnvironment`.
//!
//! This provider manages the lifecycle of custom tree views and orchestrates
//! data flow between the extension host (Cocoon) and the UI (Sky). It handles
//! registration, data dispatching, UI state updates, events, and state
//! persistence.
//!
//! ## Implementation Strategy
//!
//! The trait implementation is split across multiple helper modules for
//! maintainability:
//! - `Registration`: `RegisterTreeDataProvider`, `UnregisterTreeDataProvider`
//! - `DataAccess`: `GetChildren`, `GetTreeItem` (dispatcher logic)
//! - `UIState`: `SetTreeViewMessage`, `SetTreeViewTitle`, `SetTreeViewBadge`
//! - `Events`: `OnTreeNodeExpanded`, `OnTreeSelectionChanged`
//! - `StatePersistence`: `PersistTreeViewState`, `RestoreTreeViewState`
//! - `Visibility`: `RevealTreeItem`, `RefreshTreeView`
//!
//! The single `impl TreeViewProvider for MountainEnvironment` block in this
//! file delegates to those helper functions. This satisfies Rust's orphan rules
//! while keeping code organized.

use CommonLibrary::TreeView::TreeViewProvider::TreeViewProvider;

use async_trait::async_trait;

// Private helper modules (not re-exported)
mod Registration;

mod DataAccess;

mod UIState;

mod Events;

mod StatePersistence;

mod Visibility;

#[async_trait]
impl TreeViewProvider for crate::Environment::MountainEnvironment::MountainEnvironment {

	async fn RegisterTreeDataProvider(
		&self,

		view_identifier:String,

		options:serde_json::Value,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		Registration::register_tree_data_provider(self, view_identifier, options).await
	}

	async fn UnregisterTreeDataProvider(
		&self,

		view_identifier:String,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		Registration::unregister_tree_data_provider(self, view_identifier).await
	}

	async fn GetChildren(
		&self,

		view_identifier:String,

		element_handle:Option<String>,
	) -> Result<Vec<serde_json::Value>, CommonLibrary::Error::CommonError::CommonError> {
		DataAccess::get_children(self, view_identifier, element_handle).await
	}

	async fn GetTreeItem(
		&self,

		view_identifier:String,

		element_handle:String,
	) -> Result<serde_json::Value, CommonLibrary::Error::CommonError::CommonError> {
		DataAccess::get_tree_item(self, view_identifier, element_handle).await
	}

	async fn SetTreeViewMessage(
		&self,

		view_identifier:String,

		message:Option<String>,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		UIState::set_tree_view_message(self, view_identifier, message).await
	}

	async fn SetTreeViewTitle(
		&self,

		view_identifier:String,

		title:Option<String>,

		description:Option<String>,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		UIState::set_tree_view_title(self, view_identifier, title, description).await
	}

	async fn SetTreeViewBadge(
		&self,

		view_identifier:String,

		badge:Option<serde_json::Value>,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		UIState::set_tree_view_badge(self, view_identifier, badge).await
	}

	async fn OnTreeNodeExpanded(
		&self,

		view_identifier:String,

		element_handle:String,

		is_expanded:bool,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		Events::on_tree_node_expanded(self, view_identifier, element_handle, is_expanded).await
	}

	async fn OnTreeSelectionChanged(
		&self,

		view_identifier:String,

		selected_handles:Vec<String>,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		Events::on_tree_selection_changed(self, view_identifier, selected_handles).await
	}

	async fn PersistTreeViewState(
		&self,

		view_identifier:String,
	) -> Result<serde_json::Value, CommonLibrary::Error::CommonError::CommonError> {
		StatePersistence::persist_tree_view_state(self, view_identifier).await
	}

	async fn RestoreTreeViewState(
		&self,

		view_identifier:String,

		state_value:serde_json::Value,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		StatePersistence::restore_tree_view_state(self, view_identifier, state_value).await
	}

	async fn RevealTreeItem(
		&self,

		view_identifier:String,

		item_handle:String,

		options:serde_json::Value,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		Visibility::reveal_tree_item(self, view_identifier, item_handle, options).await
	}

	async fn RefreshTreeView(
		&self,

		view_identifier:String,

		items_to_refresh:Option<serde_json::Value>,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		Visibility::refresh_tree_view(self, view_identifier, items_to_refresh).await
	}
}
