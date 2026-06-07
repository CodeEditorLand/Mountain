//! Wire method: `localPty:freePortKillProcess`.
//! Kills whatever process is holding a TCP port so a new terminal can bind it.
//! On Unix, uses `lsof -t -i :<port>` to list PIDs then `kill -9` each one.
//! No-op on unknown port (0) or non-Unix platforms.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_u64;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {

	let Port = arg_u64(&Arguments, 0) as u16;

	if Port > 0 {
		#[cfg(unix)]
		{
			let Out = tokio::process::Command::new("lsof")
				.args(["-t", "-i", &format!(":{}", Port)])
				.output()
				.await;

			if let Ok(O) = Out {
				let Pids = String::from_utf8_lossy(&O.stdout);

				for Pid in Pids.split_whitespace() {
					if let Ok(P) = Pid.parse::<u32>() {
						let _ = tokio::process::Command::new("kill").args(["-9", &P.to_string()]).status().await;
					}
				}
			}
		}
	}

	Ok(Value::Null)
}
