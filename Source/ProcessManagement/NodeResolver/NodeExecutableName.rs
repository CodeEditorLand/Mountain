//! Platform-specific filename for the Node executable.

/// fn.
pub fn Fn() -> &'static str { if cfg!(target_os = "windows") { "node.exe" } else { "node" } }
