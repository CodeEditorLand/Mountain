//! # FeatureState
//!
//! Feature-specific state management for diagnostics, documents,
//! terminals, webviews, tree views, output channels, keybindings,
//! navigation history, and working copies.
//!
//! Each sub-module owns its own slice of the application state.
//! Access is via Arc<Mutex<...>> with short-held locks.

pub mod Debug;

pub mod Decorations;

pub mod Diagnostics;

pub mod Documents;

pub mod Keybindings;

pub mod LifecyclePhase;

pub mod Markers;

pub mod NavigationHistory;

pub mod OutputChannels;

pub mod State;

pub mod Terminals;

pub mod Tasks;

pub mod TreeViews;

pub mod Webviews;

pub mod WorkingCopy;
