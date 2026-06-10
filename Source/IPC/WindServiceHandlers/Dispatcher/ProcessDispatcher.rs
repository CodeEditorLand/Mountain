//! Process command dispatcher.

use serde_json::{Value, json};

/// Dispatches process commands.
///
/// Handled commands:
/// - `process:getPlatform`
/// - `process:getArch`
/// - `process:getPid`
/// - `process:getExecPath`
/// - `process:getMemoryInfo`
/// - `process:getCpuInfo`
pub async fn dispatch_process(command:&str) -> Result<Value, String> {
	match command {
		"process:getPlatform" => {
			Ok(json!(match std::env::consts::OS {
				"windows" => "win32",
				"macos" => "darwin",
				_ => "linux",
			}))
		},

		"process:getArch" => {
			Ok(json!(match std::env::consts::ARCH {
				"x86_64" => "x64",
				"aarch64" => "arm64",
				"x86" => "ia32",
				_ => "x64",
			}))
		},

		"process:getPid" => Ok(json!(std::process::id())),

		"process:getExecPath" => {
			Ok(json!(
				std::env::current_exe().unwrap_or_default().to_string_lossy().into_owned()
			))
		},

		"process:getMemoryInfo" => {
			Ok(json!({
				"workingSetSize": 0u64,
				"peakWorkingSetSize": 0u64,
				"privateBytes": 0u64,
				"sharedBytes": 0u64,
			}))
		},

		"process:getCpuInfo" => {
			Ok(json!([{
				"model": format!("{} ({})", std::env::consts::ARCH, std::env::consts::OS),
				"speed": 0u32,
				"times": { "user": 0u64, "nice": 0u64, "sys": 0u64, "idle": 0u64, "irq": 0u64 },
			}]))
		},

		_ => Err(format!("Unknown process command: {}", command)),
	}
}
