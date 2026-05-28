use std::{path::Path, sync::Arc};

use crate::Cache::AssetMemoryMap::Entry;

pub fn Fn(Path:&Path) -> std::io::Result<Arc<Entry::Struct>> { ::Cache::AssetMemoryMap::LoadOrInsert::Fn(Path) }
