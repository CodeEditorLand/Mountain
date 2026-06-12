//! User-interface request state. Holds pending sync UI interactions
//! (dialogs/prompts) keyed by request ID. Single child file owns the struct;
//! callers spell `UIState::UIState::State`.

/// Pending UI interaction state container.
pub mod UIState;
