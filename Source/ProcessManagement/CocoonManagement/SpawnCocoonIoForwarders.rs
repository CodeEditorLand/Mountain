//! Spawn background tasks that forward Cocoon's stdout and stderr into
//! Mountain's dev-log. Tagged lines (`[DEV:<TAG>] …`) are re-emitted under
//! the matching tag; plain lines stay under `cocoon`.
//!
//! Uses `tauri::async_runtime::spawn` (not bare `tokio::spawn`) so the tasks
//! live on the same runtime handle that Tauri owns, ensuring they are polled
//! even while the calling async task is awaiting elsewhere.

use tokio::io::{AsyncBufReadExt, BufReader};

use crate::{ProcessManagement::ExtractDevTag::Fn as ExtractDevTag, dev_log};

pub(crate) fn Fn(Process:&mut tokio::process::Child) {
	dev_log!(
		"cocoon",
		"[CocoonIO] Spawning IO forwarder tasks (stdout={}, stderr={})",
		Process.stdout.is_some(),
		Process.stderr.is_some()
	);

	if let Some(Stdout) = Process.stdout.take() {
		tauri::async_runtime::spawn(async move {
			let mut Lines = BufReader::new(Stdout).lines();

			loop {
				match Lines.next_line().await {
					Ok(Some(Line)) => {
						if let Some(Tag) = ExtractDevTag(&Line) {
							match Tag.as_str() {
								"bootstrap-stage" => dev_log!("bootstrap-stage", "[Cocoon stdout] {}", Line),
								"ext-activate" => dev_log!("ext-activate", "[Cocoon stdout] {}", Line),
								"config-prime" => dev_log!("config-prime", "[Cocoon stdout] {}", Line),
								"breaker" => dev_log!("breaker", "[Cocoon stdout] {}", Line),
								_ => dev_log!("cocoon", "[Cocoon stdout] {}", Line),
							}
						} else {
							dev_log!("cocoon", "[Cocoon stdout] {}", Line);
						}
					},
					Ok(None) => {
						dev_log!("cocoon", "[CocoonIO] stdout pipe closed (EOF)");

						break;
					},
					Err(Error) => {
						dev_log!("cocoon", "warn: [CocoonIO] stdout read error: {}", Error);

						break;
					},
				}
			}
		});
	} else {
		dev_log!("cocoon", "warn: [CocoonIO] stdout pipe not available (Stdio::piped() not set?)");
	}

	if let Some(Stderr) = Process.stderr.take() {
		tauri::async_runtime::spawn(async move {
			let mut Lines = BufReader::new(Stderr).lines();

			let mut SuppressStack = false;

			loop {
				match Lines.next_line().await {
					Ok(Some(Line)) => {
						let T = Line.trim_start();

						let IsFrame = T.starts_with("at ") || T.starts_with("code: '") || T == "}" || T.is_empty();

						if SuppressStack && IsFrame {
							dev_log!("cocoon-stderr-verbose", "[Cocoon stderr] {}", Line);

							continue;
						}

						SuppressStack = false;

						let Benign = Line.contains(": is already signed")
							|| Line.contains(": replacing existing signature")
							|| Line.contains("DeprecationWarning:")
							|| Line.contains("--trace-deprecation")
							|| Line.contains("--trace-warnings");

						let BenignHead = Line.contains("EntryNotFound (FileSystemError):")
							|| Line.contains("FileNotFound (FileSystemError):")
							|| Line.contains("[LandFix:UnhandledRejection]")
							|| Line.starts_with("[Patcher] unhandledRejection:")
							|| Line.starts_with("[Patcher] uncaughtException:");

						if BenignHead {
							SuppressStack = true;
						}

						if Benign || BenignHead {
							dev_log!("cocoon-stderr-verbose", "[Cocoon stderr] {}", Line);
						} else {
							dev_log!("cocoon", "warn: [Cocoon stderr] {}", Line);
						}
					},
					Ok(None) => {
						dev_log!("cocoon", "[CocoonIO] stderr pipe closed (EOF)");

						break;
					},
					Err(Error) => {
						dev_log!("cocoon", "warn: [CocoonIO] stderr read error: {}", Error);

						break;
					},
				}
			}
		});
	} else {
		dev_log!("cocoon", "warn: [CocoonIO] stderr pipe not available");
	}
}
