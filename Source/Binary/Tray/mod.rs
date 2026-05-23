
//! # Binary::Tray
//!
//! System tray integration for the Mountain application.
//! Manages tray icon lifecycle: initial creation (`EnableTray`),
//! theme-aware icon switching (`SwitchTrayIcon`), menu construction
//! (Open / Hide / Quit items), and window-visibility toggling on
//! left-click. Degrades gracefully if the desktop environment has no
//! tray support.
//!
//! ## Planned Work
//!
//! - Tray notification badge support
//! - Tray icon animation for background activity indication
//! - Context-menu state (enabled/disabled, checked/unchecked) per item
//! - Optimal icon sizes for HiDPI settings across platforms
//! - Platform-specific tray behavior investigation (macOS, Windows, Linux)

/// Create and register the initial system tray icon and menu.
pub mod EnableTray;

/// `#[tauri::command]` that switches the tray icon between light and dark
/// variants.
pub mod SwitchTrayIcon;
