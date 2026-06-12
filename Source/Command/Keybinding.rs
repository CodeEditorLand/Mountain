//! # Keybinding (Tauri command surface)
//!
//! Bridges keyboard-shortcut UI requests from Sky into the
//! `KeybindingProvider` registry. Five wire-bound commands, each in its
//! own file (file name = Tauri command identifier per the
//! Naming-Convention exception):
//!
//! - `GetResolvedKeybinding::GetResolvedKeybinding` - final resolved bindings
//!   after merging extension contributions + dynamic registry + user.
//! - `GetUserKeybindings::GetUserKeybindings` - user `keybindings.json`
//!   overrides (including `-command` unbind rules).
//! - `RegisterExtensionKeybindings::RegisterExtensionKeybindings` - runtime
//!   registration into `ApplicationState::Feature::Keybindings`, tagged by
//!   extension identifier.
//! - `UnregisterExtensionKeybindings::UnregisterExtensionKeybindings` - removes
//!   everything tagged with the extension identifier.
//! - `CheckKeybindingConflicts::CheckKeybindingConflicts` - normalised
//!   key-expression overlap detection (chord-aware, modifier aliasing).
//!
//! When-clause parsing/evaluation and key normalisation live in
//! `Environment::Utility::WhenClause`; context-aware resolution is the
//! `keybinding:resolve` wire method (the context-key snapshot comes from
//! the renderer).
//!
//! Errors propagate as `Result<Value, String>` for direct frontend
//! display.
//!
//! VS Code reference:
//! `vs/workbench/services/keybinding/browser/keybindingService.ts`,
//! `vs/platform/keybinding/common/keybindingResolver.ts`.
//!
//! ## Planned Work
//!
//! - Localization, custom schemes (vim/emacs/sublime)
//! - Keybinding recording, per-profile keybindings, export/import

pub mod CheckKeybindingConflicts;

pub mod GetResolvedKeybinding;

pub mod GetUserKeybindings;

pub mod RegisterExtensionKeybindings;

pub mod UnregisterExtensionKeybindings;
