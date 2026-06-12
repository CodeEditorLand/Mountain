//! # FeatureState
//!
//! Feature-specific state management for diagnostics, documents,
//! terminals, webviews, tree views, output channels, keybindings,
//! navigation history, and working copies.
//!
//! Each sub-module owns its own slice of the application state.
//! Access is via Arc<Mutex<...>> with short-held locks.

/// Debug module.
pub mod Debug;

/// Decorations module.
pub mod Decorations;

/// Diagnostics module.
pub mod Diagnostics;

/// Documents module.
pub mod Documents;

/// Keybindings module.
pub mod Keybindings;

/// Lifecyclephase module.
pub mod LifecyclePhase;

/// Markers module.
pub mod Markers;

/// Navigationhistory module.
pub mod NavigationHistory;

/// Outputchannels module.
pub mod OutputChannels;

/// State module.
pub mod State;

/// Terminals module.
pub mod Terminals;

/// Tasks module.
pub mod Tasks;

/// Treeviews module.
pub mod TreeViews;

/// Webviews module.
pub mod Webviews;

/// Workingcopy module.
pub mod WorkingCopy;
