//! `WindowStateDTO::SetZoomLevel`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct, ZoomLevel:f64) -> Result<(), String> {
		if ZoomLevel < MIN_ZOOM_LEVEL || ZoomLevel > MAX_ZOOM_LEVEL {
			return Err(format!(
				"Zoom level must be between {} and {}, got {}",
				MIN_ZOOM_LEVEL, MAX_ZOOM_LEVEL, ZoomLevel
			));
		}

		This.ZoomLevel = ZoomLevel;

		Ok(())
	}
