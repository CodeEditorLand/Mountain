//! Canonicalise without caching. For one-shot calls where the result is
//! immediately discarded - avoids polluting the cache with paths that won't be
//! repeated.

use std::path::{Path, PathBuf};

pub fn Fn(Path:&Path) -> std::io::Result<PathBuf> { dunce::canonicalize(Path) }
