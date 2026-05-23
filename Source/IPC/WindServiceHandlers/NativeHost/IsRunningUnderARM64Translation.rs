#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `nativeHost:isRunningUnderARM64Translation`.
//! On macOS checks `sysctl.proc_translated` (Rosetta 2). Cached via
//! `OnceLock` - translation status is stable for the process lifetime.

use serde_json::{Value, json};

pub async fn Fn() -> Result<Value, String> {
	#[cfg(target_os = "macos")]
	{
		// sysctl.proc_translated is stable for the process lifetime.
		static ROSETTA:std::sync::OnceLock<bool> = std::sync::OnceLock::new();

		let IsTranslated = *ROSETTA.get_or_init(|| {
			std::process::Command::new("sysctl")
				.args(["-n", "sysctl.proc_translated"])
				.output()
				.ok()
				.map(|O| String::from_utf8_lossy(&O.stdout).trim() == "1")
				.unwrap_or(false)
		});

		Ok(json!(IsTranslated))
	}

	#[cfg(not(target_os = "macos"))]
	{
		Ok(json!(false))
	}
}
