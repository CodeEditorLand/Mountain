//! Scans registered extension directories and populates the extension
//! registry state. Handles partial failures gracefully.

/// Extension cache loading from disk.
pub mod LoadFromCache;

/// Extension directory scanning and registry population.
pub mod ScanAndPopulateExtensions;
