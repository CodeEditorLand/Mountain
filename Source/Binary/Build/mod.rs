//! # Build Module
//!
//! Provides Tauri builder and plugin configuration functions.

pub mod TauriBuild;
pub mod WindowBuild;
pub mod LoggingPlugin;
pub mod LocalhostPlugin;

pub use TauriBuild::TauriBuild;
pub use WindowBuild::WindowBuild;
pub use LoggingPlugin::LoggingPlugin;
pub use LocalhostPlugin::LocalhostPlugin;
