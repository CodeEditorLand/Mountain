use std::path::{Path, PathBuf};

/// Public entry point for this module.
pub fn Fn(Path:&Path) -> std::io::Result<PathBuf> { ::Cache::PathCanon::Canonicalize::Fn(Path) }
