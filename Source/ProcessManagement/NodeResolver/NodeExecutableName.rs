//! Platform-specific filename for the Node executable.

/// Public entry point for this module.
pub fn Fn() -> &'static str { if cfg!(target_os = "windows") { "node.exe" } else { "node" } }
