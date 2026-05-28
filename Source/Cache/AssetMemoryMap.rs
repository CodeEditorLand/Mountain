//! Mountain-side compat surface for [`::Cache::AssetMemoryMap`]. Only
//! the [`LoadOrInsert`] entry-point still has direct Mountain callers
//! (`Binary::Build::Scheme`); reach for the canonical
//! `::Cache::AssetMemoryMap::*` paths elsewhere.

pub mod LoadOrInsert;
