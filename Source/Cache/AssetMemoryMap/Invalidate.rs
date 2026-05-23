
//! Drop a single cached entry. Useful for hot-reload during dev when the
//! bundler rewrites a chunk.

use std::{path::Path, sync::Arc};

use crate::Cache::AssetMemoryMap::{Entry, Map};

pub fn Fn(Path:&Path) -> Option<Arc<Entry::Struct>> { Map::Fn().remove(Path).map(|(_, V)| V) }
