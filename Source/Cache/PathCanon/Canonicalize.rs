use std::path::{Path, PathBuf};

/// fn.
pub fn Fn(Path:&Path) -> std::io::Result<PathBuf> { ::Cache::PathCanon::Canonicalize::Fn(Path) }
