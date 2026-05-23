
//! Wire method: `nativeHost:getOSStatistics`.
//! Returns Electron-shaped `{ totalmem, freemem, loadavg }` snapshot.
//! `loadavg` is the Unix triple; on Windows it's `[0,0,0]` by policy so the
//! caller gets a well-formed array.

use serde_json::{Value, json};

pub async fn Fn() -> Result<Value, String> {
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

	Ok(json!({
		"totalmem": TotalMem,
		"freemem": FreeMem,
		"loadavg": LoadAvg
	}))
}
