#![allow(non_snake_case)]

//! # Binary::Tray
//!
//! System tray integration for the Mountain application.
//! Manages tray icon lifecycle: initial creation (`EnableTray`),
//! theme-aware icon switching (`SwitchTrayIcon`), menu construction
//! (Open / Hide / Quit items), and window-visibility toggling on
//! left-click. Degrades gracefully if the desktop environment has no
//! tray support.

// TODO: add tray notification badge support
// TODO: implement tray icon animation for background activity indication
// TODO: add context-menu state (enabled/disabled, checked/unchecked) per item
// TODO: investigate optimal icon sizes for HiDPI settings across platforms
// TODO: investigate platform-specific tray behavior differences (macOS, Windows, Linux)

/// Create and register the initial system tray icon and menu.
pub mod EnableTray;

/// `#[tauri::command]` that switches the tray icon between light and dark variants.
pub mod SwitchTrayIcon;
