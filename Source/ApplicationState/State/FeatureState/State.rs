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

use std::{
	collections::HashMap,
	sync::{
		Arc,
		atomic::{AtomicU32, AtomicU64, Ordering as AtomicOrdering},
	},
};

use dashmap::DashMap;
use parking_lot::Mutex;
use tokio::sync::watch;

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
	Tasks::TaskExecutionState::TaskExecutionState,
	Terminals::TerminalState::TerminalState,
	TreeViews::TreeViewState::TreeViewState,
	Webviews::WebviewState::WebviewState,
	WorkingCopy::WorkingCopyState::WorkingCopyState,
};
use crate::dev_log;

/// Feature state combining all feature-related components.
pub struct State {
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

	/// Active task execution registry.
	pub Tasks:TaskExecutionState,

	/// Terminal instances state.
	pub Terminals:TerminalState,

	/// Tree view providers state.
	pub TreeViews:TreeViewState,

	/// Webview panels state.
	pub Webviews:WebviewState,

	/// Working-copy (dirty) state - drives the dirty dot in editor tabs.
	pub WorkingCopy:WorkingCopyState,

	/// Source-control provider handle counter, owned here
	/// (not delegated to MarkerState) so the domain boundary is explicit.
	pub SCMHandleCounter:Arc<AtomicU32>,

	/// External URI opener registrations keyed by URI scheme.
	/// Populated by `url:registerExternalUriOpener`; consulted by
	/// `nativeHost:openExternal` before falling back to the OS default.
	pub ExternalUriOpeners:Arc<Mutex<HashMap<String, ExternalUriOpenerRegistration>>>,

	/// Active text-search task abort handles, keyed by search_id.
	/// Populated when a `search:findInFiles` task is spawned;
	/// `search:cancel` looks up by ID and calls `abort()`.
	pub ActiveSearches:Arc<DashMap<u64, tokio::task::AbortHandle>>,

	/// Cooperative cancellation flags for in-flight text searches, keyed
	/// by search_id. The task-level `AbortHandle` in `ActiveSearches`
	/// only lands at an `.await` point, but the ripgrep walk inside
	/// `SearchProvider::TextSearch` is synchronous - the walker polls
	/// this flag per entry and quits early when `search:cancel` sets it.
	pub SearchCancellationFlags:Arc<DashMap<u64, Arc<std::sync::atomic::AtomicBool>>>,

	/// Monotonically increasing counter for minting search IDs.
	pub SearchIdCounter:Arc<AtomicU64>,

	/// Pending language-provider cancellation signals, keyed by the
	/// renderer-supplied request identifier. `language:cancelRequest`
	/// looks up the sender and flips it; the provider forward's
	/// `ForwardCancellable` receives the signal via `FnCancellable`
	/// and delivers `CancelOperation` on the wire.
	pub LanguageProviderCancellations:Arc<DashMap<String, watch::Sender<bool>>>,
}

/// Registration entry for a `vscode.window.registerExternalUriOpener` call.
#[derive(Clone, Debug)]
pub struct ExternalUriOpenerRegistration {
	/// URI scheme this opener handles (e.g. `"http"`, `"https"`).
	pub Scheme:String,

	/// Extension identifier that registered the opener.
	pub ExtensionId:String,

	/// Opener identifier supplied by the extension.
	pub OpenerId:String,
}

impl Default for State {
	fn default() -> Self {
		dev_log!("lifecycle", "[FeatureState::State] Initializing default feature state...");

		Self {
			Debug:Default::default(),

			Decorations:Default::default(),

			Diagnostics:Default::default(),

			Documents:Default::default(),

			Keybindings:Default::default(),

			Lifecycle:Default::default(),

			Markers:Default::default(),

			NavigationHistory:Default::default(),

			OutputChannels:Default::default(),

			Tasks:Default::default(),

			Terminals:Default::default(),

			TreeViews:Default::default(),

			Webviews:Default::default(),

			WorkingCopy:Default::default(),

			SCMHandleCounter:Arc::new(AtomicU32::new(1)),

			ExternalUriOpeners:Arc::new(Mutex::new(HashMap::new())),

			ActiveSearches:Arc::new(DashMap::new()),

			SearchCancellationFlags:Arc::new(DashMap::new()),

			SearchIdCounter:Arc::new(AtomicU64::new(1)),

			LanguageProviderCancellations:Arc::new(DashMap::new()),
		}
	}
}

impl State {
	/// Gets the next available unique identifier for a terminal instance.
	pub fn GetNextTerminalIdentifier(&self) -> u64 { self.Terminals.GetNextTerminalIdentifier() }

	/// Gets the next available unique identifier for an SCM provider.
	pub fn GetNextSourceControlManagementProviderHandle(&self) -> u32 {
		self.SCMHandleCounter.fetch_add(1, AtomicOrdering::Relaxed)
	}
}
