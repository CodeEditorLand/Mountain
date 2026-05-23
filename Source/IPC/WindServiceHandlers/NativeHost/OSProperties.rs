//! Wire method: `nativeHost:getOSProperties` (cross-platform).
//! Returns Electron-shaped `{ type, release, arch, platform, cpus }` tuple.
//! Cached for the process lifetime - OS type, version, arch, and CPU topology
//! are stable; spawning `sw_vers`/`uname` on every call wastes ~10 ms each.

use std::sync::OnceLock;

use serde_json::{Value, json};

static OS_PROPERTIES_CACHE:OnceLock<Value> = OnceLock::new();

pub async fn Fn() -> Result<Value, String> {
	if let Some(Cached) = OS_PROPERTIES_CACHE.get() {
		return Ok(Cached.clone());
	}

	let Result = compute_os_properties();

	let _ = OS_PROPERTIES_CACHE.set(Result.clone());

	Ok(Result)
}

fn compute_os_properties() -> Value {
	use sysinfo::System;

	let OsType = match std::env::consts::OS {
		"macos" => "Darwin",

		"windows" => "Windows_NT",

		"linux" => "Linux",

		_ => std::env::consts::OS,
	};

	let Release = {
		#[cfg(target_os = "macos")]
		{
			std::process::Command::new("sw_vers")
				.arg("-productVersion")
				.output()
				.ok()
				.map(|O| String::from_utf8_lossy(&O.stdout).trim().to_string())
				.unwrap_or_else(|| "14.0".to_string())
		}

		#[cfg(target_os = "windows")]
		{
			std::process::Command::new("cmd")
				.args(["/c", "ver"])
				.output()
				.ok()
				.map(|O| {
					let Output = String::from_utf8_lossy(&O.stdout);
					Output
						.split('[')
						.nth(1)
						.and_then(|S| S.split(']').next())
						.and_then(|S| S.strip_prefix("Version "))
						.unwrap_or("10.0.0")
						.to_string()
				})
				.unwrap_or_else(|| "10.0.0".to_string())
		}

		#[cfg(target_os = "linux")]
		{
			std::process::Command::new("uname")
				.arg("-r")
				.output()
				.ok()
				.map(|O| String::from_utf8_lossy(&O.stdout).trim().to_string())
				.unwrap_or_else(|| "6.1.0".to_string())
		}

		#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
		{
			"0.0.0".to_string()
		}
	};

	let mut Sys = System::new();

	Sys.refresh_cpu_all();

	let Cpus:Vec<Value> = Sys
		.cpus()
		.iter()
		.map(|Cpu| {
			json!({
				"model": Cpu.brand(),
				"speed": Cpu.frequency()
			})
		})
		.collect();

	json!({
		"type": OsType,
		"release": Release,
		"arch": std::env::consts::ARCH,
		"platform": std::env::consts::OS,
		"cpus": Cpus
	})
}
