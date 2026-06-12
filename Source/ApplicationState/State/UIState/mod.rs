//! User-interface request state. Holds pending sync UI interactions
//! (dialogs/prompts) keyed by request id. Single child file owns the struct;
//! callers spell `UIState::UIState::State`.

/// Uistate module.
pub mod UIState;
