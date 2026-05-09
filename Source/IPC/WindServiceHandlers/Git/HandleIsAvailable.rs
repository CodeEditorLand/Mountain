#![allow(non_snake_case)]

//! `localGit:isAvailable -> bool`. Cheap `git --version` probe
//! cached in a `OnceLock` for the process lifetime so the UI's
//! periodic poll doesn't re-exec git every interval.

use std::sync::OnceLock;

use serde_json::{Value, json};
use tokio::process::Command;

use crate::dev_log;

pub async fn HandleIsAvailable(_Arguments:Vec<Value>) -> Result<Value, String> {
	static CACHE:OnceLock<bool> = OnceLock::new();

	if let Some(Cached) = CACHE.get() {
		return Ok(json!(*Cached));
	}

	let Available = Command::new("git")
		.arg("--version")
		.output()
		.await
		.map(|O| O.status.success())
		.unwrap_or(false);

	let _ = CACHE.set(Available);

	dev_log!("git", "[Git] isAvailable={}", Available);

	Ok(json!(Available))
}
