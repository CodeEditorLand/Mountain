//! Canonicalise via the cache. Returns the cached entry on hit; runs
//! `dunce::canonicalize` on miss and caches the result.
//!
//! `dunce::canonicalize` is preferred over `std::fs::canonicalize` because it
//! avoids the `\\?\` UNC prefix on Windows; the underlying syscall on
//! macOS/Linux is identical (`realpath(3)`).

use std::path::{Path, PathBuf};

use crate::Cache::PathCanon::Cache::CACHE;

pub fn Fn(Path:&Path) -> std::io::Result<PathBuf> {
	if let Some(Hit) = CACHE.get(Path) {
		return Ok(Hit);
	}

	let Resolved = dunce::canonicalize(Path)?;

	CACHE.insert(Path.to_path_buf(), Resolved.clone());

	Ok(Resolved)
}
