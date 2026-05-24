pub mod New;
pub mod SetFocus;
pub mod SetVisibility;
pub mod UpdateTitle;
pub mod IsDisplayed;

use CommonLibrary::Webview::DTO::WebviewContentOptionsDTO::WebviewContentOptionsDTO;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MAX_VIEW_TYPE_LENGTH:usize = 128;

/// Maximum sidecar identifier length
const MAX_SIDECAR_IDENTIFIER_LENGTH:usize = 128;

/// Maximum extension identifier length
const MAX_EXTENSION_IDENTIFIER_LENGTH:usize = 128;

/// Maximum title length
const MAX_TITLE_LENGTH:usize = 256;

/// A struct that holds the complete state for a single Webview panel instance.
/// This is stored in `ApplicationState` to track all active Webviews managed by
/// the host.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	/// A unique UUID handle for this Webview instance.
	pub Handle:String,

	/// The view type of this Webview panel, as defined by the extension.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub ViewType:String,

	/// The current title of the Webview panel.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub Title:String,

	/// The content and security options for the Webview's content.
	pub ContentOptions:WebviewContentOptionsDTO,

	/// The options controlling the behavior of the Webview panel itself.
	// DTO: WebviewPanelOptionsDTO
	pub PanelOptions: Value,

	/// The identifier of the sidecar process that owns this Webview.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub SideCarIdentifier:String,

	/// The identifier of the extension that owns this Webview.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub ExtensionIdentifier:String,

	/// A flag indicating if the Webview panel currently has focus.
	pub IsActive:bool,

	/// A flag indicating if the Webview panel is currently visible in the UI.
	pub IsVisible:bool,
}
