//! `WindowStateDTO::GetZoomPercent`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> f64 { 100.0 + (This.ZoomLevel * 10.0) }
