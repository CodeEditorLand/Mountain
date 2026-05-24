//! `AppMenu::SetAppMenu`

use crate::dev_log;

/// No-op on non-macOS platforms - the Edit menu interception is macOS-specific.
#[cfg(not(target_os = "macos"))]
pub fn Fn(_App:&tauri::App) {}
