//! # TreeViewStateDTO
//!
//! Defines the Data Transfer Object for storing the state of a single
//! registered tree view.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::TreeView::TreeViewProvider::TreeViewProvider;

/// Holds the static options and provider for a tree view instance that has been
/// registered by an extension or natively. This is stored in `ApplicationState`
/// to track active tree views.
///
/// NOTE: This struct does not derive Serialize/Deserialize because `Arc<dyn
/// ...>` is not serializable. It is intended for in-memory state management
/// only.
#[derive(Clone)]
pub struct TreeViewStateDTO {
	/// The unique identifier for this tree view.
	pub ViewIdentifier:String,

	/// A reference to the native provider, if one exists for this view.
	/// This will be `None` for extension-provided (proxied) tree views.
	pub Provider:Option<Arc<dyn TreeViewProvider + Send + Sync>>,

	/// Whether the tree view supports selecting multiple items.
	pub CanSelectMany:bool,

	/// Whether the tree view supports drag and drop for its items.
	pub HasHandleDrag:bool,

	/// Whether the tree view supports dropping items onto it.
	pub HasHandleDrop:bool,

	/// An optional message to display in the tree view's UI.
	pub Message:Option<String>,

	/// The title of the tree view.
	pub Title:Option<String>,

	/// An optional description that appears with the title.
	pub Description:Option<String>,

	/// SideCar Identifier.
	pub SideCarIdentifier:Option<String>,
}
