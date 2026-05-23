//! Commands registered at startup and dispatched via Tauri invoke handlers.
//! Commands are grouped by domain and delegate to providers through the effect
//! system.

pub mod Bootstrap;

pub mod Hover; // Atomic structure (new)

pub mod Keybinding;

pub mod LanguageFeature;

pub mod SourceControlManagement;

pub mod TreeView;
