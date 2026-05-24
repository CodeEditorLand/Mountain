//! `WindowStateDTO::ResetZoom`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct) { This.ZoomLevel = DEFAULT_ZOOM_LEVEL; }
