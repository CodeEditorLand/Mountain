//! - Used by Mountain to track tree view provider instances
//!
//! # FIELDS
//! - ViewIdentifier: Unique tree view identifier
//! - Provider: Native Rust provider reference
//! - SideCarIdentifier: Extension sidecar host ID
//! - CanSelectMany: Multi-selection support flag
//! - HasHandleDrag: Drag-and-drop source support
//! - HasHandleDrop: Drop target support
//! - Message: Optional UI message
//! - Title: Tree view title
//! - Description: Optional description text
pub mod New;
pub mod SetMessage;
pub mod SetTitle;
pub mod SetDescription;
pub mod SetBadge;
pub mod IsNative;
pub mod IsProxy;

use std::sync::Arc;
use CommonLibrary::TreeView::TreeViewProvider::TreeViewProvider;

/// Maximum view identifier length
const MAX_VIEW_IDENTIFIER_LENGTH:usize = 128;

/// Maximum sidecar identifier length
const MAX_SIDECAR_IDENTIFIER_LENGTH:usize = 128;

/// Maximum message length
const MAX_MESSAGE_LENGTH:usize = 1024;

/// Maximum title length
const MAX_TITLE_LENGTH:usize = 256;

/// Maximum description length
const MAX_DESCRIPTION_LENGTH:usize = 512;

/// Maximum badge length (serialized JSON)
const MAX_BADGE_LENGTH:usize = 2048;

/// Holds the static options and provider for a tree view instance that has been
/// registered by an extension or natively. This is stored in `ApplicationState`
/// to track active tree views.
/// This struct holds references to either a native (Rust) provider or metadata
/// for a proxied (extension) provider.
/// NOTE: This struct does not derive Serialize/Deserialize because `Arc<dyn
/// ...>` is not serializable. It is intended for in-memory state management
/// only.
#[derive(Clone)]
pub struct Struct {
	/// The unique identifier for this tree view.
	pub ViewIdentifier:String,

	/// A reference to the native provider, if one exists for this view.
	/// This will be `None` for extension-provided (proxied) tree views.
	pub Provider:Option<Arc<dyn TreeViewProvider + Send + Sync>>,

	/// The identifier of the sidecar process that hosts the provider logic.
	/// This will be `Some` for extension-provided (proxied) tree views.
	pub SideCarIdentifier:Option<String>,

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

	/// Badge to display on the tree view (typically a count or string)
	pub Badge:Option<String>,
}
