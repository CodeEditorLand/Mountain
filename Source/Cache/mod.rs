
//! Mountain caching primitives.
//!
//! - [`self::AssetMemoryMap`] - file-backed mmap cache for the bundled
//!   workbench assets (and any other static-disk asset served via
//!   the `vscode-file://` / `tauri://` / `land://` schemes).
//! - [`self::PathCanon`] - process-wide canonical-path cache; collapses repeat
//!   `dunce::canonicalize` calls used by the fs-scope security gate.
//!
//! All entries here are additive performance helpers; the editor
//! continues to function with any one of them disabled.

pub mod AssetMemoryMap;

pub mod PathCanon;
