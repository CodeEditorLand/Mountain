
//! Process-wide canonical-path cache.
//!
//! Keyed by lexical input path; value is the result of `dunce::canonicalize`.
//! Hits skip the syscall; misses run it and cache the result.
//!
//! The fs-scope security gate (`Environment::Utility::PathSecurity`)
//! canonicalises every incoming path on every call. The same paths recur
//! thousands of times during boot - 113 extension manifest paths, ~80 chunked
//! workbench JS imports, ~60 git-extension scope checks, every
//! `vscode-file://` request - so collapsing repeats to a hash lookup saves
//! ~150 ms cumulative on the boot path.
//!
//! TimeToLive = 60 s to bound staleness against external `mv`/rename. moka's
//! `time_to_idle` resets on each access, so hot paths stay cached
//! indefinitely while one-shot paths evict naturally.

pub mod CacheStats;

pub mod Canonicalize;

pub mod CanonicalizeUncached;

pub mod Clear;

pub mod Invalidate;

pub mod SpawnDiagnosticLogger;

pub mod Stats;

pub(crate) mod Cache;
