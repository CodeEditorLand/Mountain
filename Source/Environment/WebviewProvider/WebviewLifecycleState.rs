#![allow(non_snake_case)]

//! Lifecycle state of a webview panel. Roughly mirrors the VS Code
//! webview state machine (Unloaded → Loading → Loaded → Visible /
//! Hidden → Disposed).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enum {
	Unloaded,
	Loading,
	Loaded,
	Visible,
	Hidden,
	Disposed,
}
