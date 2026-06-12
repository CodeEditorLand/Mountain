use std::{path::Path, sync::Arc};

/// fn.
pub fn Fn(Path:&Path) -> std::io::Result<Arc<::Cache::AssetMemoryMap::Entry::Struct>> {
	::Cache::AssetMemoryMap::LoadOrInsert::Fn(Path)
}
