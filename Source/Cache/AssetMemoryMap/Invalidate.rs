use std::{path::Path, sync::Arc};

use crate::Cache::AssetMemoryMap::Entry;

pub fn Fn(Path:&Path) -> Option<Arc<Entry::Struct>> { ::Cache::AssetMemoryMap::Invalidate::Fn(Path) }
