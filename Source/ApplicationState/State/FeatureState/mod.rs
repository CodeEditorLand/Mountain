//! # FeatureState
//!
//! Feature-specific state management for diagnostics, documents,
//! terminals, webviews, tree views, output channels, keybindings,
//! navigation history, decorations, debug state, lifecycle phase,
//! markers, tasks, and working copies.
//!
//! Each sub-module owns a slice of the application state. Access is
//! via `Arc<Mutex<...>>` with short-held locks.
//!
//! ## Sub-modules
//!
//! - [`Debug`]: Debug session state
//! - [`Decorations`]: Text editor decoration state
//! - [`Diagnostics`]: Diagnostic markers state
//! - [`Documents`]: Document state
//! - [`Keybindings`]: Keybinding state
//! - [`LifecyclePhase`]: Lifecycle phase tracking
//! - [`Markers`]: General marker state
//! - [`NavigationHistory`]: Navigation history state
//! - [`OutputChannels`]: Output channel state
//! - [`State`]: Aggregate state container
//! - [`Tasks`]: Task execution state
//! - [`Terminals`]: Terminal instance state
//! - [`TreeViews`]: Tree view panel state
//! - [`Webviews`]: Webview panel state
//! - [`WorkingCopy`]: Working copy (unsaved/backup) state

/// Debug session state.
pub mod Debug;

/// Text editor decoration state.
pub mod Decorations;

/// Diagnostic markers state.
pub mod Diagnostics;

/// Document state.
pub mod Documents;

/// Keybinding state.
pub mod Keybindings;

/// Lifecycle phase tracking.
pub mod LifecyclePhase;

/// General marker state.
pub mod Markers;

/// Navigation history state.
pub mod NavigationHistory;

/// Output channel state.
pub mod OutputChannels;

/// Aggregate state container for FeatureState sub-states.
pub mod State;

/// Terminal instance state.
pub mod Terminals;

/// Task execution state.
pub mod Tasks;

/// Tree view panel state.
pub mod TreeViews;

/// Webview panel state.
pub mod Webviews;

/// Working copy (unsaved/backup) state.
pub mod WorkingCopy;
