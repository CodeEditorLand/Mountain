//! Load `Path` into the cache (or return the existing entry).
//!
//! Returns `Err` only if the file cannot be opened or memory-mapped; missing
//! brotli siblings are silently ignored (best-effort optimisation).

use std::{
	path::{Path, PathBuf},
	sync::Arc,
};

use memmap2::Mmap;

use crate::{
	Cache::AssetMemoryMap::{Entry, Map, MimeFromExtension},
	dev_log,
};

pub fn Fn(Path:&Path) -> std::io::Result<Arc<Entry::Struct>> {
	if let Some(Existing) = Map::Fn().get(Path) {
		return Ok(Existing.clone());
	}

	let File = std::fs::File::open(Path)?;

	let Length = File.metadata()?.len() as usize;

	// SAFETY: caller agrees the file is not truncated underneath us for the
	// lifetime of the MemoryMap. The bundle directory is read-only at runtime;
	// mutations happen at build time and require a binary restart.
	let Mapping = unsafe { Mmap::map(&File)? };

	let BrotliPath = {
		let mut B = Path.as_os_str().to_owned();

		B.push(".br");

		PathBuf::from(B)
	};

	let Brotli = std::fs::File::open(&BrotliPath)
		.ok()
		.and_then(|F| unsafe { Mmap::map(&F).ok() });

	let Mime = MimeFromExtension::Fn(Path);

	let MarkerEntry = Arc::new(Entry::Struct { Mapping, Mime, Length, Brotli });

	dev_log!(
		"asset-cache",
		"mmap insert path={} bytes={} brotli={}",
		Path.display(),
		Length,
		MarkerEntry.Brotli.is_some()
	);

	Map::Fn().insert(Path.to_path_buf(), MarkerEntry.clone());

	Ok(MarkerEntry)
}
