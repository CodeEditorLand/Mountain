//! `WebviewStateDTO::SetVisibility`

use super::Struct;
use CommonLibrary::Webview::DTO::WebviewContentOptionsDTO::WebviewContentOptionsDTO;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn Fn(This:&mut Struct, IsVisible:bool) { This.IsVisible = IsVisible; }
