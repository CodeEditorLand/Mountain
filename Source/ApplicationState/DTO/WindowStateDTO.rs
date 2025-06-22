//! # WindowStateDTO
//!
//! Defines the Data Transfer Object for storing the state of the main
//! application window.

use serde::{Deserialize, Serialize};

/// Holds information about the state of the main application window, such as
/// whether it is focused or fullscreen, and its current zoom level.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct WindowStateDTO {
	#[serde(default)]
	pub IsFocused:bool,

	#[serde(default)]
	pub IsFullScreen:bool,

	#[serde(default = "DefaultZoomLevel")]
	pub ZoomLevel:f64,
}

fn DefaultZoomLevel() -> f64 { 0.0 }
