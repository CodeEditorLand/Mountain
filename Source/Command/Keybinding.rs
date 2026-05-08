#![allow(non_snake_case)]

//! # Keybinding (Tauri command surface)
//!
//! Bridges keyboard-shortcut UI requests from Sky into the
//! `KeybindingProvider` registry. Five wire-bound commands, each in its
//! own file (file name = Tauri command identifier per the
//! Naming-Convention exception):
//!
//! - `GetResolvedKeybinding::GetResolvedKeybinding` - final resolved bindings
//!   after merging defaults + extension contributions + user.
//! - `GetUserKeybindings::GetUserKeybindings` - user overrides (stub).
//! - `RegisterExtensionKeybindings::RegisterExtensionKeybindings` (stub).
//! - `UnregisterExtensionKeybindings::UnregisterExtensionKeybindings` (stub).
//! - `CheckKeybindingConflicts::CheckKeybindingConflicts` - chord overlap
//!   detection (stub).
//!
//! Errors propagate as `Result<Value, String>` for direct frontend
//! display.
//!
//! VS Code reference:
//! `vs/workbench/services/keybinding/browser/keybindingService.ts`,
//! `vs/platform/keybinding/common/keybindingResolver.ts`.
//!
//! TODO: weighted resolution (user > extension > default), persistence
//! to ApplicationState, when-clause contexts, chord (multi-stroke)
//! sequences, platform-specific bindings, conflict-resolution UI.

pub mod CheckKeybindingConflicts;

pub mod GetResolvedKeybinding;

pub mod GetUserKeybindings;

pub mod RegisterExtensionKeybindings;

pub mod UnregisterExtensionKeybindings;
