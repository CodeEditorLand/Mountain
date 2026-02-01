//! # WebViewStateDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for WebView panel state
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to track WebView lifecycle
//!
//! # FIELDS
/// - Handle: Unique WebView UUID
/// - ViewType: Extension-defined view type
/// - Title: Current panel title
/// - ContentOptions: Web content and security settings
/// - PanelOptions: Panel behavior options
/// - SideCarIdentifier: Host sidecar process ID
/// - ExtensionIdentifier: Owner extension ID
/// - IsActive: Focus state flag
/// - IsVisible: Visibility state flag
use CommonLibrary::WebView::DTO::WebViewContentOptionsDTO::WebViewContentOptionsDTO;
use serde::{Deserialize, Serialize};
// For PanelOptions, etc.
use serde_json::Value;

/// Maximum handle length (UUID string)
const MAX_HANDLE_LENGTH:usize = 128;

/// Maximum view type length
const MAX_VIEW_TYPE_LENGTH:usize = 128;

/// Maximum sidecar identifier length
const MAX_SIDECAR_IDENTIFIER_LENGTH:usize = 128;

/// Maximum extension identifier length
const MAX_EXTENSION_IDENTIFIER_LENGTH:usize = 128;

/// Maximum title length
const MAX_TITLE_LENGTH:usize = 256;

/// A struct that holds the complete state for a single WebView panel instance.
/// This is stored in `ApplicationState` to track all active WebViews managed by
/// the host.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WebViewStateDTO {
	/// A unique UUID handle for this WebView instance.
	pub Handle:String,

	/// The view type of this WebView panel, as defined by the extension.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub ViewType:String,

	/// The current title of the WebView panel.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub Title:String,

	/// The content and security options for the WebView's content.
	pub ContentOptions:WebViewContentOptionsDTO,

	/// The options controlling the behavior of the WebView panel itself.
	// DTO: WebViewPanelOptionsDTO
	pub PanelOptions: Value,

	/// The identifier of the sidecar process that owns this WebView.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub SideCarIdentifier:String,

	/// The identifier of the extension that owns this WebView.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub ExtensionIdentifier:String,

	/// A flag indicating if the WebView panel currently has focus.
	pub IsActive:bool,

	/// A flag indicating if the WebView panel is currently visible in the UI.
	pub IsVisible:bool,
}

impl WebViewStateDTO {
	/// Creates a new WebViewStateDTO with validation.
	///
	/// # Arguments
	/// * `Handle` - Unique WebView handle
	/// * `ViewType` - Extension-defined view type
	/// * `Title` - Panel title
	/// * `ContentOptions` - Web content options
	/// * `PanelOptions` - Panel behavior options
	/// * `SideCarIdentifier` - Sidecar process ID
	/// * `ExtensionIdentifier` - Extension ID
	///
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn New(
		Handle:String,
		ViewType:String,
		Title:String,
		ContentOptions:WebViewContentOptionsDTO,
		PanelOptions:Value,
		SideCarIdentifier:String,
		ExtensionIdentifier:String,
	) -> Result<Self, String> {
		// Validate handle length
		if Handle.len() > MAX_HANDLE_LENGTH {
			return Err(format!("Handle exceeds maximum length of {} bytes", MAX_HANDLE_LENGTH));
		}

		// Validate view type length
		if ViewType.len() > MAX_VIEW_TYPE_LENGTH {
			return Err(format!("ViewType exceeds maximum length of {} bytes", MAX_VIEW_TYPE_LENGTH));
		}

		// Validate title length
		if Title.len() > MAX_TITLE_LENGTH {
			return Err(format!("Title exceeds maximum length of {} bytes", MAX_TITLE_LENGTH));
		}

		// Validate sidecar identifier length
		if SideCarIdentifier.len() > MAX_SIDECAR_IDENTIFIER_LENGTH {
			return Err(format!(
				"SideCar identifier exceeds maximum length of {} bytes",
				MAX_SIDECAR_IDENTIFIER_LENGTH
			));
		}

		// Validate extension identifier length
		if ExtensionIdentifier.len() > MAX_EXTENSION_IDENTIFIER_LENGTH {
			return Err(format!(
				"Extension identifier exceeds maximum length of {} bytes",
				MAX_EXTENSION_IDENTIFIER_LENGTH
			));
		}

		Ok(Self {
			Handle,
			ViewType,
			Title,
			ContentOptions,
			PanelOptions,
			SideCarIdentifier,
			ExtensionIdentifier,
			IsActive:false,
			IsVisible:false,
		})
	}

	/// Updates the focus state of the WebView.
	///
	/// # Arguments
	/// * `IsActive` - New focus state
	pub fn SetFocus(&mut self, IsActive:bool) { self.IsActive = IsActive; }

	/// Updates the visibility state of the WebView.
	///
	/// # Arguments
	/// * `IsVisible` - New visibility state
	pub fn SetVisibility(&mut self, IsVisible:bool) { self.IsVisible = IsVisible; }

	/// Updates the WebView title with validation.
	///
	/// # Arguments
	/// * `Title` - New title
	///
	/// # Returns
	/// Result indicating success or error if title too long
	pub fn UpdateTitle(&mut self, Title:String) -> Result<(), String> {
		if Title.len() > MAX_TITLE_LENGTH {
			return Err(format!("Title exceeds maximum length of {} bytes", MAX_TITLE_LENGTH));
		}

		self.Title = Title;
		Ok(())
	}

	/// Checks if the WebView is currently displayed (visible and focused).
	pub fn IsDisplayed(&self) -> bool { self.IsVisible || self.IsActive }
}
