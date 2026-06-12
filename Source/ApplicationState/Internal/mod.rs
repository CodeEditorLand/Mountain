//! Private utilities for ApplicationState: extension scanning, persistence,
//! recovery, serialization, and text processing.
//!
//! ## Sub-modules
//!
//! - [`ExtensionScanner`]: Extension directory scanning and caching
//! - [`PathResolution`]: Memento storage path resolution
//! - [`Persistence`]: State persistence (memento load/save)
//! - [`Recovery`]: State recovery on corruption
//! - [`Serialization`]: URL-based serialization/deserialization
//! - [`TextProcessing`]: Text line and EOL analysis

/// Extension directory scanning and cache management.
pub mod ExtensionScanner;

/// Memento storage path resolution.
pub mod PathResolution;

/// State persistence: memento loading and saving.
pub mod Persistence;

/// State recovery and validation after corruption.
pub mod Recovery;

/// URL-based serialization and deserialization.
pub mod Serialization;

/// Text line and end-of-line analysis.
pub mod TextProcessing;
