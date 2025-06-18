// @module WindowStateDTO
// @description Defines the Data Transfer Object for storing the state of the
// main application window.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};

// Holds information about the state of the main application window, such as
// whether it is focused or fullscreen.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct WindowStateDTO {
	#[serde(default)]
	pub IsFocused:bool,
	#[serde(default)]
	pub IsFullScreen:bool,
	#[serde(default = "default_zoom")]
	pub ZoomLevel:f64,
}

fn default_zoom() -> f64 { 0.0 }
