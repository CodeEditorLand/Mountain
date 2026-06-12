//! Wire method: `nativeHost:getOSStatistics`.
//! Returns Electron-shaped `{ totalmem, freemem, loadavg }` snapshot.
//! `loadavg` is the Unix triple; on Windows it's `[0,0,0]` by policy so the
//! caller gets a well-formed array.
//! Cached with a 10 s TTL - memory/load figures move slowly and allocating a
//! fresh `sysinfo::System` plus a memory refresh on every call wastes async
//! pool time.

use std::{
	sync::{Mutex, OnceLock},
	time::{Duration, Instant},
};

use serde_json::{Value, json};

static OS_STATISTICS_CACHE:OnceLock<Mutex<Option<(Instant, Value)>>> = OnceLock::new();

const CACHE_TTL:Duration = Duration::from_secs(10);

pub async fn Fn() -> Result<Value, String> {
	let Cache = OS_STATISTICS_CACHE.get_or_init(|| Mutex::new(None));

	if let Ok(Guard) = Cache.lock() {
		if let Some((ComputedAt, Cached)) = Guard.as_ref() {
			if ComputedAt.elapsed() < CACHE_TTL {
				return Ok(Cached.clone());
			}
		}
	}

	let Result = compute_os_statistics();

	if let Ok(mut Guard) = Cache.lock() {
		*Guard = Some((Instant::now(), Result.clone()));
	}

	Ok(Result)
}

fn compute_os_statistics() -> Value {
	use sysinfo::System;

	let mut Sys = System::new();

	Sys.refresh_memory();

	let TotalMem = Sys.total_memory();

	let FreeMem = Sys.available_memory();

	let LoadAvg = {
		#[cfg(unix)]
		{
			let Load = System::load_average();

			vec![Load.one, Load.five, Load.fifteen]
		}

		#[cfg(not(unix))]
		{
			vec![0.0, 0.0, 0.0]
		}
	};

	json!({
		"totalmem": TotalMem,
		"freemem": FreeMem,
		"loadavg": LoadAvg
	})
}
