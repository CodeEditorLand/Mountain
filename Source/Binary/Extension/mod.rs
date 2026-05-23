
//! # Binary::Extension
//!
//! Extension startup utilities called from `Binary::Main::AppLifecycle`.
//! Configures extension scan paths and populates the initial extension
//! registry before the workbench finishes loading.

/// Resolve and register the extension scan-path list from config and defaults.
pub mod ScanPathConfigure;

/// Walk the scan paths, parse manifests, and populate the extension registry.
pub mod ExtensionPopulate;
