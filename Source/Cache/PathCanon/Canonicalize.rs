use std::path::{Path, PathBuf};

pub fn Fn(Path:&Path) -> std::io::Result<PathBuf> { ::Cache::PathCanon::Canonicalize::Fn(Path) }
