//! # WebViewStateDTO
//!
//! Defines the Data Transfer Object for storing the state of a single active
//! WebView panel.

#![allow(non_snake_case, non_camel_case_types)]

use Common::WebView::DTO::WebViewContentOptionsDTO::WebViewContentOptionsDTO;
use serde::{Deserialize, Serialize};
// For PanelOptions, etc.
use serde_json::Value;

/// A struct that holds the complete state for a single WebView panel instance.
/// This is stored in `ApplicationState` to track all active WebViews managed by
/// the host.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WebViewStateDTO {
	/// A unique UUID handle for this WebView instance.
	pub Handle:String,

	/// The view type of this WebView panel, as defined by the extension.
	pub ViewType:String,

	/// The current title of the WebView panel.
	pub Title:String,

	/// The content and security options for the WebView's content.
	pub ContentOptions:WebViewContentOptionsDTO,

	/// The options controlling the behavior of the WebView panel itself.
	// DTO: WebViewPanelOptionsDTO
	pub PanelOptions: Value,

	/// The identifier of the sidecar process that owns this WebView.
	pub SidecarIdentifier:String,

	/// The identifier of the extension that owns this WebView.
	pub ExtensionIdentifier:String,

	/// A flag indicating if the WebView panel currently has focus.
	pub IsActive:bool,

	/// A flag indicating if the WebView panel is currently visible in the UI.
	pub IsVisible:bool,
}
