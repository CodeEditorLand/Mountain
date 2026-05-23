
//! Application self-update. Tauri's bundled updater + optional Air gRPC
//! delegation. Currently dormant - zero call sites; kept atomized for the
//! eventual Help → Check for Updates wire-up.

pub mod UpdateService;
