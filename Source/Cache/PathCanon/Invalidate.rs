//! Force-evict an entry. Called from `notify` watchers when a path rename is
//! observed inside the workspace, or by the dev-mode hot-reload signal.

use std::path::Path;

use crate::Cache::PathCanon::Cache::CACHE;

pub fn Fn(Path:&Path) { CACHE.invalidate(Path); }
