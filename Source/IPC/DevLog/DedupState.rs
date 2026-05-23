
//! Consecutive-duplicate suppression buffer used by the
//! `dev_log!` macro under `Trace=short`. Holds the last logged
//! key + a repeat count; the macro flushes a `(xN)` tail when
//! the key changes.

use std::sync::Mutex;

pub struct Struct {
	pub LastKey:String,

	pub Count:u64,
}

pub static DEDUP:Mutex<Struct> = Mutex::new(Struct { LastKey:String::new(), Count:0 });
