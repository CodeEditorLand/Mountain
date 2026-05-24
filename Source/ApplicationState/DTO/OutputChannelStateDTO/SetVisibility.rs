//! `OutputChannelStateDTO::SetVisibility`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&mut Struct, IsVisible:bool) { This.IsVisible = IsVisible; }
