//! Wire method: `nativeHost:startPowerSaveBlocker`.
//!
//! Allocates a monotonically incrementing blocker ID. The blocker is
//! tracked entirely on the front-end; Mountain returns the ID.

use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

pub fn Fn() -> Result<Value, String> {
	static NEXT_BLOCKER_ID:AtomicU64 = AtomicU64::new(1);

	let Id = NEXT_BLOCKER_ID.fetch_add(1, Ordering::Relaxed);

	Ok(json!(Id))
}
