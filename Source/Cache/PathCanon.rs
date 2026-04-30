#![allow(non_snake_case)]

//! Process-wide canonical-path cache.
//!
//! Keyed by the lexical input path; value is the result of
//! `dunce::canonicalize`. On a hit the syscall is skipped; on a
//! miss the syscall runs and the result is cached.
//!
//! The fs-scope security gate (`PathSecurity::IsPathAllowedForAccess`)
//! canonicalises every incoming path on every call. The same paths
//! recur thousands of times during boot - 113 extension manifest
//! paths, ~80 chunked workbench JS imports, ~60 git-extension scope
//! checks, every `vscode-file://` request - so collapsing repeats
//! to a hash lookup saves ~150 ms cumulative on the boot path.
//!
//! TTL = 60 s to bound staleness against external `mv`/rename. moka's
//! `time_to_idle` resets on each access, so hot paths stay cached
//! indefinitely while one-shot paths evict naturally.

use std::{
	path::{Path, PathBuf},
	time::Duration,
};

use moka::sync::Cache;
use once_cell::sync::Lazy;

use crate::dev_log;

static CACHE:Lazy<Cache<PathBuf, PathBuf>> = Lazy::new(|| {
	Cache::builder()
		.max_capacity(8192)
		.time_to_idle(Duration::from_secs(60))
		.build()
});

/// Canonicalise via the cache. Returns the cached entry on hit;
/// runs `dunce::canonicalize` on miss and caches the result.
///
/// `dunce::canonicalize` is preferred over `std::fs::canonicalize`
/// because it avoids the `\\?\` UNC prefix on Windows; the underlying
/// syscall on macOS/Linux is identical (`realpath(3)`).
pub fn Canonicalize(Path:&Path) -> std::io::Result<PathBuf> {
	if let Some(Hit) = CACHE.get(Path) {
		return Ok(Hit);
	}
	let Resolved = dunce::canonicalize(Path)?;
	CACHE.insert(Path.to_path_buf(), Resolved.clone());
	Ok(Resolved)
}

/// Canonicalise without caching. For one-shot calls where the
/// result is immediately discarded - avoids polluting the cache
/// with paths that won't be repeated.
pub fn CanonicalizeUncached(Path:&Path) -> std::io::Result<PathBuf> { dunce::canonicalize(Path) }

/// Force-evict an entry. Called from `notify` watchers when a path
/// rename is observed inside the workspace, or by the dev-mode
/// hot-reload signal.
pub fn Invalidate(Path:&Path) { CACHE.invalidate(Path); }

/// Clear the entire cache. Diagnostic / shutdown use.
pub fn Clear() { CACHE.invalidate_all(); }

/// Diagnostic snapshot.
pub fn Stats() -> CacheStats {
	CacheStats {
		Entries:CACHE.entry_count() as usize,
		WeightedSize:CACHE.weighted_size() as usize,
	}
}

#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
	pub Entries:usize,
	pub WeightedSize:usize,
}

/// Spawn a tokio task that logs cache stats every 30 s under the
/// `path-canon` trace tag. Optional; call from `RunTime::Setup`
/// when the user has `Trace=path-canon` enabled.
pub fn SpawnDiagnosticLogger() {
	tokio::spawn(async {
		let mut Interval = tokio::time::interval(Duration::from_secs(30));
		Interval.tick().await; // skip the immediate first tick
		loop {
			Interval.tick().await;
			let Snapshot = Stats();
			dev_log!("path-canon", "entries={} weighted={}", Snapshot.Entries, Snapshot.WeightedSize);
		}
	});
}
