pub fn Matches(MethodName:&str) -> bool { MethodName == "$gitExec" }

use std::time::Duration;

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$gitExec" => {
			crate::effect!(_run_time, {
				let (Args, WorkingDir) = if let Some(Object) = Parameters.as_object() {
					let ArgsVec:Vec<String> = Object
						.get("args")
						.and_then(Value::as_array)
						.map(|Array| Array.iter().filter_map(|V| V.as_str().map(str::to_string)).collect())
						.unwrap_or_default();

					let RepoPath = Object
						.get("repository")
						.or_else(|| Object.get("cwd"))
						.and_then(Value::as_str)
						.map(str::to_string)
						.unwrap_or_default();

					(ArgsVec, RepoPath)
				} else {
					let ArgsVec:Vec<String> = Parameters
						.get(0)
						.and_then(Value::as_array)
						.map(|Array| Array.iter().filter_map(|V| V.as_str().map(str::to_string)).collect())
						.unwrap_or_default();

					let RepoPath = Parameters
						.get(1)
						.and_then(Value::as_str)
						.map(str::to_string)
						.unwrap_or_default();

					(ArgsVec, RepoPath)
				};

				let Cwd = if WorkingDir.is_empty() {
					std::env::current_dir().unwrap_or_default()
				} else {
					std::path::PathBuf::from(&WorkingDir)
				};

				dev_log!(
					"grpc",
					"[$gitExec] Received gRPC Request: Method='$gitExec' args={:?} cwd={}",
					Args,
					Cwd.display()
				);

				let StartAt = std::time::Instant::now();

				let OutputResult = tokio::time::timeout(
					Duration::from_secs(30),
					tokio::process::Command::new("git").args(&Args).current_dir(&Cwd).output(),
				)
				.await
				.map_err(|_| format!("$gitExec timed out after 30s: args={:?} cwd={}", Args, Cwd.display()))?
				.map_err(|Error| format!("$gitExec failed to spawn git: {}", Error))?;

				let ExitCode = OutputResult.status.code().unwrap_or(-1);

				let Stdout = String::from_utf8_lossy(&OutputResult.stdout).to_string();

				let Stderr = String::from_utf8_lossy(&OutputResult.stderr).to_string();

				dev_log!(
					"grpc",
					"[$gitExec] exit={} elapsed={}ms stdout={}B stderr={}B",
					ExitCode,
					StartAt.elapsed().as_millis(),
					Stdout.len(),
					Stderr.len()
				);

				Ok(json!({
					"exitCode": ExitCode,
					"stdout": Stdout,
					"stderr": Stderr,
				}))
			})
		},

		_ => None,
	}
}
