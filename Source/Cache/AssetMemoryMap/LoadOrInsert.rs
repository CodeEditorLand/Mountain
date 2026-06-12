use std::{path::Path, sync::Arc};

/// Public entry point for this module.
pub fn Fn(Path:&Path) -> std::io::Result<Arc<::Cache::AssetMemoryMap::Entry::Struct>> {
	::Cache::AssetMemoryMap::LoadOrInsert::Fn(Path)
}
