//! `WindowStateDTO::ZoomOut`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct, Step:f64) -> Result<(), String> {
		let NewZoom = This.ZoomLevel - Step;

		This.SetZoomLevel(NewZoom)
	}
