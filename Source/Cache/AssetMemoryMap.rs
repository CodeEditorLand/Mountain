#![allow(non_snake_case)]

//! Memory-mapped asset cache for the bundled workbench (and any
//! other static-disk asset served via the `vscode-file://`,
//! `tauri://`, or `land://` scheme handlers).
//!
//! # Why mmap and not `Vec<u8>`
//!
//! The bundled workbench under `Element/Sky/Target/Static/Application/`
//! ships ~80 MB of `.js`, `.css`, `.svg`, and font assets. The
//! WKWebView fetches each on cold load via the scheme handler. Two
//! patterns hurt:
//!
//! 1. `std::fs::read(path)` per request - each handler invocation pays a
//!    `read(2)` + allocation + memcpy of the full file.
//! 2. The legacy `LRU<String, Vec<u8>>` cache duplicates the OS page cache:
//!    memory is held twice (once by the kernel as file-backed pages, once by
//!    Rust as anonymous heap).
//!
//! `memmap2::Mmap` gives us the file-backed bytes addressable via a
//! single page-table lookup. The OS handles paging; we hand the
//! webview a borrowed slice. RSS attribution moves from "anonymous
//! heap" to "file-backed pages" - which the OS evicts under
//! pressure rather than swapping.
//!
//! # Brotli sibling
//!
//! Each entry transparently picks up an optional `<file>.br`
//! sibling produced by the brotli pre-bake script
//! (`Maintain/Build/Brotli/Pre-Bake.ts`). The scheme handler can
//! query `Entry::AsBrotliSlice()` and serve the pre-compressed
//! bytes with `Content-Encoding: br` when the client offers it.
//!
//! # Concurrency
//!
//! `DashMap` shards are lock-free for read; insertion uses one
//! shard's lock. Multiple webviews requesting the same path race
//! once on first load; after that all reads are wait-free.
//!
//! # Eviction
//!
//! None today. Process is a desktop app; the working set is
//! bounded by the bundle size (~80 MB). If memory pressure becomes
//! a concern, swap `DashMap` for `moka::sync::Cache` and set a TTI
//! of 5 minutes.

use std::{
	path::{Path, PathBuf},
	sync::{Arc, OnceLock},
};

use dashmap::DashMap;
use memmap2::Mmap;

use crate::dev_log;

/// One mmap entry. Holds the file-backed mapping plus metadata
/// computed once at load time.
pub struct Entry {
	/// The mmap itself. Keep alive as long as any webview body
	/// references it.
	pub Mapping:Mmap,
	/// Cached MIME from the file extension. Avoids the match arm
	/// on the hot path.
	pub Mime:&'static str,
	/// File size at mmap time. Used for `Content-Length`.
	pub Length:usize,
	/// Optional pre-brotli-compressed sibling (path with `.br`
	/// suffix). `None` if no sibling exists at load time.
	pub Brotli:Option<Mmap>,
}

impl Entry {
	/// Borrow the entire mapping as a slice. Caller responsibility
	/// to keep the `Arc<Entry>` alive for the lifetime of any
	/// response body that captures the slice.
	pub fn AsSlice(&self) -> &[u8] { &self.Mapping[..] }

	/// Borrow the brotli-precompressed sibling if present.
	pub fn AsBrotliSlice(&self) -> Option<&[u8]> { self.Brotli.as_ref().map(|M| &M[..]) }

	/// Length of the brotli sibling, if present. Useful for
	/// `Content-Length` when serving the precompressed payload.
	pub fn BrotliLength(&self) -> Option<usize> { self.Brotli.as_ref().map(|M| M.len()) }
}

/// Global cache. Lazily initialised on first request.
fn Map() -> &'static DashMap<PathBuf, Arc<Entry>> {
	static MAP:OnceLock<DashMap<PathBuf, Arc<Entry>>> = OnceLock::new();
	MAP.get_or_init(DashMap::new)
}

/// Load `Path` into the cache (or return the existing entry).
///
/// Returns `Err` only if the file cannot be opened or mmap'd; missing
/// brotli siblings are silently ignored (best-effort optimisation).
pub fn LoadOrInsert(Path:&Path) -> std::io::Result<Arc<Entry>> {
	if let Some(Existing) = Map().get(Path) {
		return Ok(Existing.clone());
	}

	let File = std::fs::File::open(Path)?;
	let Metadata = File.metadata()?;
	let Length = Metadata.len() as usize;

	// SAFETY: caller agrees the file is not truncated underneath us
	// for the lifetime of the mmap. In Land, the bundle directory
	// is read-only at runtime; mutations happen at build time and
	// require a binary restart.
	let Mapping = unsafe { Mmap::map(&File)? };

	// Prefer brotli-precompressed sibling if present.
	let BrotliPath = {
		let mut B = Path.as_os_str().to_owned();
		B.push(".br");
		PathBuf::from(B)
	};
	let Brotli = std::fs::File::open(&BrotliPath)
		.ok()
		.and_then(|F| unsafe { Mmap::map(&F).ok() });

	let Mime = MimeFromExtension(Path);

	let MarkerEntry = Arc::new(Entry { Mapping, Mime, Length, Brotli });

	dev_log!(
		"asset-cache",
		"mmap insert path={} bytes={} brotli={}",
		Path.display(),
		Length,
		MarkerEntry.Brotli.is_some()
	);

	Map().insert(Path.to_path_buf(), MarkerEntry.clone());
	Ok(MarkerEntry)
}

/// Drop a single cached entry. Useful for hot-reload during dev when
/// the bundler rewrites a chunk.
pub fn Invalidate(Path:&Path) -> Option<Arc<Entry>> { Map().remove(Path).map(|(_, V)| V) }

/// Clear the entire cache. Called on shutdown or on an explicit
/// flush signal.
pub fn Clear() { Map().clear(); }

/// Snapshot of cache stats for diagnostics.
pub fn Stats() -> CacheStats {
	let mut Bytes = 0usize;
	let mut Entries = 0usize;
	let mut BrotliEntries = 0usize;
	let mut BrotliBytes = 0usize;
	for Reference in Map().iter() {
		Entries += 1;
		Bytes += Reference.value().Length;
		if let Some(BLength) = Reference.value().BrotliLength() {
			BrotliEntries += 1;
			BrotliBytes += BLength;
		}
	}
	CacheStats { Entries, BrotliEntries, Bytes, BrotliBytes }
}

#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
	pub Entries:usize,
	pub BrotliEntries:usize,
	pub Bytes:usize,
	pub BrotliBytes:usize,
}

/// Map a file extension to its IANA media type. Mirror of the
/// `MimeFromExtension` helper in `Binary/Build/Scheme.rs` so the
/// cache layer is self-contained; the existing public function
/// in `Scheme.rs` continues to serve the inline path until that
/// site is migrated to consume `Entry::Mime` directly.
fn MimeFromExtension(Path:&Path) -> &'static str {
	match Path.extension().and_then(|S| S.to_str()).unwrap_or("") {
		"js" | "mjs" | "cjs" => "application/javascript; charset=utf-8",
		"css" => "text/css; charset=utf-8",
		"html" | "htm" => "text/html; charset=utf-8",
		"json" => "application/json; charset=utf-8",
		"map" => "application/json; charset=utf-8",
		"svg" => "image/svg+xml",
		"png" => "image/png",
		"jpg" | "jpeg" => "image/jpeg",
		"gif" => "image/gif",
		"webp" => "image/webp",
		"woff" => "font/woff",
		"woff2" => "font/woff2",
		"ttf" => "font/ttf",
		"otf" => "font/otf",
		"wasm" => "application/wasm",
		"ico" => "image/x-icon",
		"txt" => "text/plain; charset=utf-8",
		"md" => "text/markdown; charset=utf-8",
		_ => "application/octet-stream",
	}
}
