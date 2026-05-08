#![allow(non_snake_case)]

//! Capture a diagnostic snapshot of the cache.

use crate::Cache::PathCanon::{Cache::CACHE, CacheStats};

pub fn Fn() -> CacheStats::Struct {
	CacheStats::Struct {
		Entries:CACHE.entry_count() as usize,

		WeightedSize:CACHE.weighted_size() as usize,
	}
}
