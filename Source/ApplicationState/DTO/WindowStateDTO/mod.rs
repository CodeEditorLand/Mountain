pub mod New;
pub mod SetZoomLevel;
pub mod ZoomIn;
pub mod ZoomOut;
pub mod ResetZoom;
pub mod GetZoomPercent;

use serde::{Deserialize, Serialize};

/// Holds information about the state of the main application window, such as
/// whether it is focused or fullscreen, and its current zoom level.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	/// Whether the window currently has input focus
	#[serde(default)]
	pub IsFocused:bool,

	/// Whether the window is in fullscreen mode
	#[serde(default)]
	pub IsFullScreen:bool,

	/// Zoom level for content scaling (typically -10 to 10)
	#[serde(default = "DefaultZoomLevel")]
	pub ZoomLevel:f64,
}
