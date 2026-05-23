//! Tauri command - return process memory in `{ private, shared,
//! residentSet }` form (bytes). Per-platform: `ps` on macOS, `tasklist`
//! on Windows, `/proc/self/statm` on Linux. Errors fall back to zero
//! triple so the renderer keeps working in the rare case the platform
//! probe fails.

use serde_json::{Value, json};

#[tauri::command]
pub async fn process_get_memory_info() -> Result<Value, String> {
	#[cfg(target_os = "macos")]
	{
		let Output = std::process::Command::new("ps")
			.args(["-o", "rss=,vsz=", "-p", &std::process::id().to_string()])
			.output();

		match Output {
			Ok(Out) => {
				let Text = String::from_utf8_lossy(&Out.stdout);

				let Parts:Vec<&str> = Text.split_whitespace().collect();

				let Rss = Parts.first().and_then(|V| V.parse::<u64>().ok()).unwrap_or(0) * 1024;

				let _Vsz = Parts.get(1).and_then(|V| V.parse::<u64>().ok()).unwrap_or(0) * 1024;

				Ok(json!({ "private": Rss, "shared": 0, "residentSet": Rss }))
			},

			Err(_) => Ok(json!({ "private": 0, "shared": 0, "residentSet": 0 })),
		}
	}

	#[cfg(target_os = "windows")]
	{
		let Output = std::process::Command::new("tasklist")
			.args(["/FI", &format!("PID eq {}", std::process::id()), "/FO", "CSV", "/NH"])
			.output();

		match Output {
			Ok(Out) => {
				let Text = String::from_utf8_lossy(&Out.stdout);

				let MemStr = Text.split(',').nth(4).unwrap_or("\"0 K\"");

				let MemKb:u64 = MemStr
					.chars()
					.filter(|C| C.is_ascii_digit())
					.collect::<String>()
					.parse()
					.unwrap_or(0);

				let MemBytes = MemKb * 1024;

				Ok(json!({ "private": MemBytes, "shared": 0, "residentSet": MemBytes }))
			},

			Err(_) => Ok(json!({ "private": 0, "shared": 0, "residentSet": 0 })),
		}
	}

	#[cfg(target_os = "linux")]
	{
		match tokio::fs::read_to_string("/proc/self/statm").await {
			Ok(Content) => {
				let Parts:Vec<&str> = Content.split_whitespace().collect();

				let PageSize:u64 = 4096;

				let _Vsz = Parts.first().and_then(|V| V.parse::<u64>().ok()).unwrap_or(0) * PageSize;

				let Rss = Parts.get(1).and_then(|V| V.parse::<u64>().ok()).unwrap_or(0) * PageSize;

				let Shared = Parts.get(2).and_then(|V| V.parse::<u64>().ok()).unwrap_or(0) * PageSize;

				Ok(json!({
					"private": Rss.saturating_sub(Shared),
					"shared": Shared,
					"residentSet": Rss
				}))
			},

			Err(_) => Ok(json!({ "private": 0, "shared": 0, "residentSet": 0 })),
		}
	}

	#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
	{
		Ok(json!({ "private": 0, "shared": 0, "residentSet": 0 }))
	}
}
