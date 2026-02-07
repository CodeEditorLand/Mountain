//! # WindowStateDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for main window state
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to track window presentation state
//!
//! # FIELDS
//! - IsFocused: Window focus state
//! - IsFullScreen: Fullscreen mode state
//! - ZoomLevel: Window zoom level
use serde::{Deserialize, Serialize};

/// Minimum allowed zoom level
const MIN_ZOOM_LEVEL:f64 = -20.0;

/// Maximum allowed zoom level
const MAX_ZOOM_LEVEL:f64 = 20.0;

/// Default zoom level
const DEFAULT_ZOOM_LEVEL:f64 = 0.0;

/// Holds information about the state of the main application window, such as
/// whether it is focused or fullscreen, and its current zoom level.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct WindowStateDTO {
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

impl WindowStateDTO {
	/// Creates a new WindowStateDTO with validation.
	///
	/// # Arguments
	/// * `IsFocused` - Focus state
	/// * `IsFullScreen` - Fullscreen state
	/// * `ZoomLevel` - Zoom level
	///
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn New(IsFocused:bool, IsFullScreen:bool, ZoomLevel:f64) -> Result<Self, String> {
		// Validate zoom level range
		if ZoomLevel < MIN_ZOOM_LEVEL || ZoomLevel > MAX_ZOOM_LEVEL {
			return Err(format!(
				"Zoom level must be between {} and {}, got {}",
				MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL, ZoomLevel
			));
		}

		Ok(Self { IsFocused, IsFullScreen, ZoomLevel })
	}

	/// Sets the zoom level with validation.
	///
	/// # Arguments
	/// * `ZoomLevel` - New zoom level
	///
	/// # Returns
	/// Result indicating success or error if out of range
	pub fn SetZoomLevel(&mut self, ZoomLevel:f64) -> Result<(), String> {
		if ZoomLevel < MIN_ZOOM_LEVEL || ZoomLevel > MAX_ZOOM_LEVEL {
			return Err(format!(
				"Zoom level must be between {} and {}, got {}",
				MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL, ZoomLevel
			));
		}

		self.ZoomLevel = ZoomLevel;
		Ok(())
	}

	/// Increases the zoom level by a step.
	///
	/// # Arguments
	/// * `Step` - Zoom increment amount
	///
	/// # Returns
	/// Result indicating success or error if would exceed range
	pub fn ZoomIn(&mut self, Step:f64) -> Result<(), String> {
		let NewZoom = self.ZoomLevel + Step;
		self.SetZoomLevel(NewZoom)
	}

	/// Decreases the zoom level by a step.
	///
	/// # Arguments
	/// * `Step` - Zoom decrement amount
	///
	/// # Returns
	/// Result indicating success or error if would exceed range
	pub fn ZoomOut(&mut self, Step:f64) -> Result<(), String> {
		let NewZoom = self.ZoomLevel - Step;
		self.SetZoomLevel(NewZoom)
	}

	/// Resets the zoom level to default.
	pub fn ResetZoom(&mut self) { self.ZoomLevel = DEFAULT_ZOOM_LEVEL; }

	/// Gets the current zoom level as a percentage.
	/// A zoom level of 0 corresponds to 100%.
	pub fn GetZoomPercent(&self) -> f64 { 100.0 + (self.ZoomLevel * 10.0) }
}

fn DefaultZoomLevel() -> f64 { DEFAULT_ZOOM_LEVEL }
