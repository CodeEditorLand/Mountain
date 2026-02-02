//! # TreeViewStateDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for tree view state
//! - In-memory state tracking (not serializable due to trait object)
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

/// Holds the static options and provider for a tree view instance that has been
/// registered by an extension or natively. This is stored in `ApplicationState`
/// to track active tree views.
///
/// This struct holds references to either a native (Rust) provider or metadata
/// for a proxied (extension) provider.
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
}

impl TreeViewStateDTO {
	/// Creates a new TreeViewStateDTO with validation.
	///
	/// # Arguments
	/// * `ViewIdentifier` - Unique view identifier
	/// * `Provider` - Optional native provider
	/// * `SideCarIdentifier` - Optional sidecar identifier
	/// * `CanSelectMany` - Multi-selection support
	/// * `HasHandleDrag` - Drag support
	/// * `HasHandleDrop` - Drop support
	///
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn New(
		ViewIdentifier:String,
		Provider:Option<Arc<dyn TreeViewProvider + Send + Sync>>,
		SideCarIdentifier:Option<String>,
		CanSelectMany:bool,
		HasHandleDrag:bool,
		HasHandleDrop:bool,
	) -> Result<Self, String> {
		// Validate view identifier length
		if ViewIdentifier.len() > MAX_VIEW_IDENTIFIER_LENGTH {
			return Err(format!(
				"View identifier exceeds maximum length of {} bytes",
				MAX_VIEW_IDENTIFIER_LENGTH
			));
		}

		// Validate sidecar identifier length
		if let Some(SideCarID) = &SideCarIdentifier {
			if SideCarID.len() > MAX_SIDECAR_IDENTIFIER_LENGTH {
				return Err(format!(
					"SideCar identifier exceeds maximum length of {} bytes",
					MAX_SIDECAR_IDENTIFIER_LENGTH
				));
			}
		}

		Ok(Self {
			ViewIdentifier,
			Provider,
			SideCarIdentifier,
			CanSelectMany,
			HasHandleDrag,
			HasHandleDrop,
			Message:None,
			Title:None,
			Description:None,
		})
	}

	/// Sets the UI message with validation.
	///
	/// # Arguments
	/// * `Message` - Message text
	///
	/// # Returns
	/// Result indicating success or error if message too long
	pub fn SetMessage(&mut self, Message:String) -> Result<(), String> {
		if Message.len() > MAX_MESSAGE_LENGTH {
			return Err(format!("Message exceeds maximum length of {} bytes", MAX_MESSAGE_LENGTH));
		}

		self.Message = Some(Message);
		Ok(())
	}

	/// Sets the title with validation.
	///
	/// # Arguments
	/// * `Title` - Title text
	///
	/// # Returns
	/// Result indicating success or error if title too long
	pub fn SetTitle(&mut self, Title:String) -> Result<(), String> {
		if Title.len() > MAX_TITLE_LENGTH {
			return Err(format!("Title exceeds maximum length of {} bytes", MAX_TITLE_LENGTH));
		}

		self.Title = Some(Title);
		Ok(())
	}

	/// Sets the description with validation.
	///
	/// # Arguments
	/// * `Description` - Description text
	///
	/// # Returns
	/// Result indicating success or error if description too long
	pub fn SetDescription(&mut self, Description:String) -> Result<(), String> {
		if Description.len() > MAX_DESCRIPTION_LENGTH {
			return Err(format!(
				"Description exceeds maximum length of {} bytes",
				MAX_DESCRIPTION_LENGTH
			));
		}

		self.Description = Some(Description);
		Ok(())
	}

	/// Checks if this is a native (Rust) tree view.
	pub fn IsNative(&self) -> bool { self.Provider.is_some() }

	/// Checks if this is a proxy (extension) tree view.
	pub fn IsProxy(&self) -> bool { self.SideCarIdentifier.is_some() }
}
