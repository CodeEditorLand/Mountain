
//! Platform-specific filename for the Node executable.

pub fn Fn() -> &'static str { if cfg!(target_os = "windows") { "node.exe" } else { "node" } }
