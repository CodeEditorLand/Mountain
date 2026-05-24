//! components:
//! - Diagnostics: Diagnostic errors state
//! - Documents: Open documents state
//! - Terminals: Terminal instances state
//! - Webviews: Webview panels state
//! - TreeViews: Tree view providers state
//! - OutputChannels: Output channel state
//! - Markers: Marker-related state
//!
//! ## KEY COMPONENTS
//! - State: Main struct combining all feature state
//! - Default: Initialization implementation
//!
//! ## ERROR HANDLING
//! - Thread-safe access via `Arc<Mutex<...>>`
//! - Proper lock error handling with `MapLockError` helpers
//!
//! ## LOGGING
//! State changes are logged at appropriate levels (debug, info, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly and release immediately
//! - Avoid nested locks to prevent deadlocks
//! - Use Arc for shared ownership across threads
//!
//! ## TODO
//! - [ ] Add feature state validation invariants
//! - [ ] Implement feature lifecycle events
//! - [ ] Add feature state metrics collection
pub mod GetNextTerminalIdentifier;
pub mod GetNextSourceControlManagementProviderHandle;

use super::{
	Debug::DebugState::DebugState,
	Decorations::DecorationsState::DecorationsState,
	Diagnostics::DiagnosticsState::DiagnosticsState,
	Documents::DocumentState::DocumentState,
	Keybindings::KeybindingState::KeybindingState,
	LifecyclePhase::LifecyclePhaseState::LifecyclePhaseState,
	Markers::MarkerState::MarkerState,
	NavigationHistory::NavigationHistoryState::NavigationHistoryState,
	OutputChannels::OutputChannelState::OutputChannelState,
	Terminals::TerminalState::TerminalState,
	TreeViews::TreeViewState::TreeViewState,
	Webviews::WebviewState::WebviewState,
	WorkingCopy::WorkingCopyState::WorkingCopyState,
};
use crate::dev_log;

/// Feature state combining all feature-related components.
#[derive(Clone)]
pub struct Struct {
	/// Debug provider state.
	pub Debug:DebugState,

	/// File/folder decoration state (git badges, error squiggles, custom
	/// badges).
	pub Decorations:DecorationsState,

	/// Diagnostic errors state.
	pub Diagnostics:DiagnosticsState,

	/// Open documents state.
	pub Documents:DocumentState,

	/// Dynamic keybinding registry.
	pub Keybindings:KeybindingState,

	/// Application lifecycle phase state.
	pub Lifecycle:LifecyclePhaseState,

	/// Editor navigation history (back/forward stack).
	pub NavigationHistory:NavigationHistoryState,

	/// Marker-related state.
	pub Markers:MarkerState,

	/// Output channel state.
	pub OutputChannels:OutputChannelState,

	/// Terminal instances state.
	pub Terminals:TerminalState,

	/// Tree view providers state.
	pub TreeViews:TreeViewState,

	/// Webview panels state.
	pub Webviews:WebviewState,

	/// Working-copy (dirty) state - drives the dirty dot in editor tabs.
	pub WorkingCopy:WorkingCopyState,
}
