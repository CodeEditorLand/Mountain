//! `localGit:isAvailable -> bool`. Cheap `git --version` probe
//! cached in a `OnceLock` for the process lifetime so the UI's
//! periodic poll doesn't re-exec git every interval.

use std::sync::OnceLock;

use serde_json::{Value, json};
use tokio::process::Command;

use crate::dev_log;

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> {
	// Cache only a `true` result - once git is confirmed available it stays
	// available for the process lifetime.  A `false` result is NOT cached:
	// the first probe may run before EnhanceShellEnvironment has extended
	// PATH, so a transient miss must not permanently disable the SCM UI.
	static CACHE:OnceLock<bool> = OnceLock::new();

	if CACHE.get() == Some(&true) {
		return Ok(json!(true));
	}

	let Available = Command::new("git")
		.arg("--version")
		.output()
		.await
		.map(|O| O.status.success())
		.unwrap_or(false);

	if Available {
		let _ = CACHE.set(true);
	}

	dev_log!("git", "[Git] isAvailable={}", Available);

	Ok(json!(Available))
}
