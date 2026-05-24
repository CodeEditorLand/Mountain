//! `WindowStateDTO::New`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(IsFocused:bool, IsFullScreen:bool, ZoomLevel:f64) -> Result<Self, String> {
		// Validate zoom level range
		if ZoomLevel < MIN_ZOOM_LEVEL || ZoomLevel > MAX_ZOOM_LEVEL {
			return Err(format!(
				"Zoom level must be between {} and {}, got {}",
				MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL, ZoomLevel
			));
		}

		Ok(Self { IsFocused, IsFullScreen, ZoomLevel })
	}
