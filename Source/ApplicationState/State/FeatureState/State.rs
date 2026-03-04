//! # State Module (FeatureState)
//!
//! ## RESPONSIBILITIES
//! Combines all feature-related state components into a single state struct.
//!
//! ## ARCHITECTURAL ROLE
//! State is the main composite struct that combines all FeatureState
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

use log::debug;

use super::{
	Debug::DebugState::DebugState,
	Diagnostics::DiagnosticsState::DiagnosticsState,
	Documents::DocumentState::DocumentState,
	Markers::MarkerState::MarkerState,
	OutputChannels::OutputChannelState::OutputChannelState,
	Terminals::TerminalState::TerminalState,
	TreeViews::TreeViewState::TreeViewState,
	Webviews::WebviewState::WebviewState,
};

/// Feature state combining all feature-related components.
#[derive(Clone)]
pub struct State {
	/// Debug provider state.
	pub Debug:DebugState,

	/// Diagnostic errors state.
	pub Diagnostics:DiagnosticsState,

	/// Open documents state.
	pub Documents:DocumentState,

	/// Terminal instances state.
	pub Terminals:TerminalState,

	/// Webview panels state.
	pub Webviews:WebviewState,

	/// Tree view providers state.
	pub TreeViews:TreeViewState,

	/// Output channel state.
	pub OutputChannels:OutputChannelState,

	/// Marker-related state.
	pub Markers:MarkerState,
}

impl Default for State {
	fn default() -> Self {
		debug!("[FeatureState::State] Initializing default feature state...");

		Self {
			Debug:Default::default(),
			Diagnostics:Default::default(),
			Documents:Default::default(),
			Terminals:Default::default(),
			Webviews:Default::default(),
			TreeViews:Default::default(),
			OutputChannels:Default::default(),
			Markers:Default::default(),
		}
	}
}

impl State {
	/// Gets the next available unique identifier for a terminal instance.
	pub fn GetNextTerminalIdentifier(&self) -> u64 { self.Terminals.GetNextTerminalIdentifier() }

	/// Gets the next available unique identifier for an SCM provider.
	pub fn GetNextSourceControlManagementProviderHandle(&self) -> u32 {
		self.Markers.GetNextSourceControlManagementProviderHandle()
	}
}
