//! Process-global canonical-path cache backing store.

use std::{path::PathBuf, time::Duration};

use moka::sync::Cache;
use once_cell::sync::Lazy;

pub static CACHE:Lazy<Cache<PathBuf, PathBuf>> = Lazy::new(|| {
	Cache::builder()
		.max_capacity(8192)
		.time_to_idle(Duration::from_secs(60))
		.build()
});
