// @module WebViewStateDTO
// @description Defines the Data Transfer Object for storing the state of a
// single active webview panel.
//

#![allow(non_snake_case, non_camel_case_types)]

use Common::webview::DTO::{WebViewContentOptionsDTO, WebViewPanelOptionsDTO};
use serde::{Deserialize, Serialize};

// A struct that holds the complete state for a single webview panel instance.
// This is stored in `ApplicationState` to track all active webviews managed by
// the host.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WebViewStateDTO {
	// A unique UUID handle for this webview instance.
	pub Handle:String,

	// The view type of this webview panel, as defined by the extension.
	pub ViewType:String,

	// The current title of the webview panel.
	pub Title:String,

	// The content and security options for the webview's content.
	pub ContentOptions:WebViewContentOptionsDTO,

	// The options controlling the behavior of the webview panel itself.
	pub PanelOptions:WebViewPanelOptionsDTO,

	// The identifier of the sidecar process that owns this webview.
	pub SidecarIdentifier:String,

	// The identifier of the extension that owns this webview.
	pub ExtensionId:String,

	// A flag indicating if the webview panel currently has focus.
	pub IsActive:bool,

	// A flag indicating if the webview panel is currently visible in the User Interface.
	pub IsVisible:bool,
}
