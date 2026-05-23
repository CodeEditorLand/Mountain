
//! Memory-mapped asset cache for the bundled workbench (and any other
//! static-disk asset served via `vscode-file://`, `tauri://`, or `land://`
//! scheme handlers).
//!
//! ## Why MemoryMap and not `Vec<u8>`
//!
//! The bundled workbench under `Element/Sky/Target/Static/Application/` ships
//! ~80 MB of `.js`, `.css`, `.svg`, and font assets. Per-request `fs::read`
//! pays a `read(2)` + alloc + memcpy of the full file; a `LRU<String,
//! Vec<u8>>` cache duplicates the kernel page cache (memory held twice).
//! `memmap2::Mmap` hands the webview a borrowed slice of file-backed pages
//! that the OS evicts under pressure.
//!
//! ## Brotli sibling
//!
//! Each entry transparently picks up an optional `<file>.br` produced by
//! `Maintain/Build/Brotli/Pre-Bake.ts`. Scheme handlers can serve the
//! pre-compressed bytes with `Content-Encoding: br` when the client offers it.
//!
//! ## Concurrency / eviction
//!
//! `DashMap` shards are wait-free for read; first-load races on one shard
//! lock. No eviction today - bundle is bounded by ~80 MB.

pub mod CacheStats;

pub mod Clear;

pub mod Entry;

pub mod Invalidate;

pub mod LoadOrInsert;

pub mod Stats;

pub(crate) mod Map;

pub(crate) mod MimeFromExtension;
