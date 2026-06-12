//! Lifecycle state of a webview panel. Roughly mirrors the VS Code
//! webview state machine (Unloaded → Loading → Loaded → Visible /
//! Hidden → Disposed).

use serde::{Deserialize, Serialize};

/// Lifecycle state of a webview panel.
///
/// Mirrors the VS Code webview state machine: Unloaded → Loading →
/// Loaded → Visible / Hidden → Disposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enum {
	/// Webview has been created but no content is loaded yet.
	Unloaded,

	/// Webview content is being fetched or prepared.
	Loading,

	/// Webview has finished loading its content.
	Loaded,

	/// Webview is currently visible to the user.
	Visible,

	/// Webview exists but is hidden from view.
	Hidden,

	/// Webview has been fully torn down and disposed.
	Disposed,
}
